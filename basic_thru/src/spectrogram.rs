use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::vg::{ImageId, Paint, Path, PixelFormat, ImageFlags}; // femtovg
use std::sync::Arc;
use std::cell::RefCell;

pub struct Spectrogram {
    // 外部から渡されるデータ受信機
    #[allow(clippy::type_complexity)]
    consumer: RefCell<ringbuf::Consumer<Vec<f32>, Arc<ringbuf::HeapRb<Vec<f32>>>>>,
    
    // 描画用の内部状態
    image_id: RefCell<Option<ImageId>>, // GPUテクスチャID
    pixel_buffer: RefCell<Vec<u8>>,     // CPU側のRGBAバッファ
    
    // サイズ設定
    width: usize,
    height: usize,
}

impl Spectrogram {
    pub fn new(cx: &mut Context, consumer: ringbuf::Consumer<Vec<f32>, Arc<ringbuf::HeapRb<Vec<f32>>>>) -> Handle<'_, Self> {
        // 例: 幅512px, 高さ256px のスペクトログラム
        // GUI上の表示サイズとは独立した「内部解像度」
        let width = 512;
        let height = 256;
        
        // 黒で初期化 (RGBA = 4 bytes per pixel)
        let buffer = vec![0; width * height * 4];

        Self {
            consumer: RefCell::new(consumer),
            image_id: RefCell::new(None),
            pixel_buffer: RefCell::new(buffer),
            width,
            height,
        }
        .build(cx, |_| {}) // Viewとしてビルド
    }

    /// ヘルパー: dB値を色(RGBA)に変換
    fn magnitude_to_color(mag: f32) -> (u8, u8, u8, u8) {
        // 1. 振幅を加算して安定させる (0除算防止)
        let db = 20.0 * (mag + 1e-6).log10();
        
        // 2. 表示レンジの設定 (例: -60dB ~ 0dB)
        let min_db = -60.0; 
        let max_db = 0.0;
        
        // 3. 0.0 ~ 1.0 に正規化
        let norm = ((db - min_db) / (max_db - min_db)).clamp(0.0, 1.0);
        
        // 4. 色変換 (ヒートマップ: 黒->青->赤->黄)
        let r = (norm * 255.0) as u8;
        let b = ((1.0 - norm) * 255.0) as u8;
        let g = if norm > 0.8 { ((norm - 0.8) * 5.0 * 255.0) as u8 } else { 0 }; 
        
        (r, g, b, 255)
    }

    /// 内部バッファの更新処理
    /// 新しいスペクトルデータがあれば、画像を左に1列シフトして右端に描く
    fn update_buffer(&self) -> bool {
        let mut consumer = self.consumer.borrow_mut();
        let mut dirty = false;
        
        // キューに溜まっている分を全て処理して、最新の状態まで進める（あるいは全部描く）
        // ここでは「溜まっている分だけスクロール」させることで、高速な更新にも追従させる
        // ただし、1フレームで進みすぎると見にくいので制限をかけてもいい
        
        let mut _processed_some = false;
        
        // バッファを借用
        let mut buffer = self.pixel_buffer.borrow_mut();
        
        // Max 4 columns per frame to prevent freezing if queue is full
        let max_cols = 4;
        let mut cols = 0;

        while let Some(spectrum) = consumer.pop() {
            dirty = true;
            _processed_some = true;
            
            // 1. 画像全体を左に1ピクセルずらす
            let _row_bytes = self.width * 4;
            // メモリコピーによるシフト
            for y in 0..self.height {
                let row_start = y * self.width * 4;
                let row_end = row_start + (self.width * 4);
                // 最初の4バイト(左端のピクセル)は上書きされて消える
                // copy_within(src_range, dest_start)
                buffer.copy_within((row_start + 4)..row_end, row_start);
            }

            // 2. 右端 (x = width - 1) に新しいデータを書き込む
            // spectrum は周波数ビンごとの配列 (サイズは fft_size/2 くらい)
            // height (256) に合わせてリサンプリング
            let spectrum: &Vec<f32> = &spectrum;
            let spec_len = spectrum.len();
            if spec_len > 0 {
                for y in 0..self.height {
                    // Y座標反転（下を低音にするため）: 0(bottom) -> 0Hz
                    // femtovg coordinate: 0 is top.
                    // So y=0 (top) should be High Freq? Usually yes.
                    // But spectrogram convention: bottom is low freq.
                    // height-1 (bottom) -> 0Hz.
                    
                    // Simple linear mapping for now (Log is better but linear is easier to verify bins)
                    let bin_index = (self.height - 1 - y) * spec_len / self.height;
                    
                    let mag = if bin_index < spec_len {
                        spectrum[bin_index]
                    } else {
                        0.0
                    };
                    
                    let (r, g, b, a) = Self::magnitude_to_color(mag);
                    
                    let pixel_idx = (y * self.width + (self.width - 1)) * 4;
                    buffer[pixel_idx] = r;
                    buffer[pixel_idx + 1] = g;
                    buffer[pixel_idx + 2] = b;
                    buffer[pixel_idx + 3] = a;
                }
            }
            
            cols += 1;
            if cols >= max_cols {
                break;
            }
        }
        
        // もしキューがつまってたら最新まで飛ばすべきだが、リングバッファなのでConsumer側でDrainしない限り残る。
        // ここでは最大4列描いたら抜けることにする（残りは次のフレームで）。
        
        dirty
    }
}

impl View for Spectrogram {
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        // 1. データの更新
        let dirty = self.update_buffer();

        // 2. GPUテクスチャの作成または更新
        let mut image_id = self.image_id.borrow_mut();
        
        if image_id.is_none() {
            // 初回作成
            let id = canvas.create_image_empty(
                self.width, 
                self.height, 
                PixelFormat::Rgba8, 
                ImageFlags::NEAREST
            ).expect("Failed to create spectrogram texture");
            *image_id = Some(id);
        }

        if let Some(id) = *image_id {
            if dirty {
                let buffer = self.pixel_buffer.borrow();
                // Create an ImageSource from the raw buffer with correct pixel format
                let rgba_buffer: &[nih_plug_vizia::vizia::vg::rgb::RGBA8] = unsafe {
                    std::slice::from_raw_parts(buffer.as_ptr() as *const _, buffer.len() / 4)
                };
                let img_ref = nih_plug_vizia::vizia::vg::imgref::ImgRef::new(
                    rgba_buffer,
                    self.width,
                    self.height
                );
                canvas.update_image(id, img_ref, 0, 0)
                    .unwrap(); // update_image returns Result
            }

            // 3. 描画 (矩形塗りつぶし)
            let bounds = cx.bounds();
            
            // ImagePaint
            // x, y, w, h, angle, alpha
            let paint = Paint::image(id, bounds.x, bounds.y, bounds.w, bounds.h, 0.0, 1.0);
            
            let mut path = Path::new();
            path.rect(bounds.x, bounds.y, bounds.w, bounds.h);
            canvas.fill_path(&path, &paint);
        }
    }
}
