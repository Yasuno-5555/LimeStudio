//! VST3 Window Attachment (The Bridge)
//! 
//! "Confidence is the product. Visual clarity is the weapon."
//! DAWのウィンドウハンドル(HWND/NSView/X11)にWGPUを安全に割り込ませるための最下層ブリッジ。

use raw_window_handle::{
    RawWindowHandle, RawDisplayHandle, HasRawWindowHandle, HasRawDisplayHandle,
    HandleError,
};
use std::sync::Arc;

/// ホストから渡された生のウィンドウハンドルを保持するラッパー
pub struct ExternalWindowHandle {
    pub raw_window: RawWindowHandle,
    pub raw_display: RawDisplayHandle,
}

unsafe impl HasRawWindowHandle for ExternalWindowHandle {
    fn raw_window_handle(&self) -> Result<RawWindowHandle, HandleError> {
        Ok(self.raw_window)
    }
}

unsafe impl HasRawDisplayHandle for ExternalWindowHandle {
    fn raw_display_handle(&self) -> Result<RawDisplayHandle, HandleError> {
        Ok(self.raw_display)
    }
}

/// WGPU Surfaceを外部ハンドルから構築する
pub async fn create_surface_from_host(
    instance: &wgpu::Instance,
    window_handle: ExternalWindowHandle,
) -> Result<wgpu::Surface<'static>, wgpu::CreateSurfaceError> {
    // ライフタイム 'static のために、ハンドルをArc等で管理する必要がある場合がある
    // ここでは unsafe を用いて、ホストウィンドウが生きている間のみ有効なSurfaceを作成する。
    // (プラグインのUIライフサイクル管理が重要)
    
    // Safety: 呼び出し側（VST3 Editor）がウィンドウの生存期間を保証する必要がある。
    unsafe {
        instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: window_handle.raw_display,
            raw_window_handle: window_handle.raw_window,
        })
    }
}

/// 汎用的なアタッチメント情報
pub struct AttachmentConfig {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
}

/// A marker for the attachment layer
pub struct Vst3Attachment;
