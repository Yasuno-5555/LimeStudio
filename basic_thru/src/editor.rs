use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::{Arc, Mutex};

use crate::BasicThruParams;

/// エディタの状態（GUIスレッドで保持される）
pub struct _EditorState {
    // スペクトルデータ受信用のConsumer
    // ViziaのEventLoop内でこれをポーリングして描画データを更新する想定
    // Mutexでラップしているのは、Optionを取り出すときや移動時のため（基本は単一スレッドアクセスだがViziaStateの制約）
    #[allow(clippy::type_complexity)]
    pub monitor_consumer:
        Arc<Mutex<Option<ringbuf::Consumer<Vec<f32>, Arc<ringbuf::HeapRb<Vec<f32>>>>>>>,
}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (800, 600))
}

use crate::spectrogram::Spectrogram;

#[allow(clippy::type_complexity)]
pub(crate) fn create_editor(
    _params: Arc<BasicThruParams>,
    monitor_consumer: Option<ringbuf::Consumer<Vec<f32>, Arc<ringbuf::HeapRb<Vec<f32>>>>>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    let consumer_cell = Mutex::new(monitor_consumer);

    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        // Assets or Fonts setup if needed

        let consumer = consumer_cell.lock().unwrap().take();

        // Basic Layout
        VStack::new(cx, |cx| {
            Label::new(cx, "Limestudio Wavelet Engine")
                .font_size(30.0)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(20.0));

            // Spectrogram Area
            if let Some(c) = consumer {
                Spectrogram::new(cx, c)
                    .height(Stretch(1.0))
                    .width(Stretch(1.0));
            } else {
                Label::new(cx, "No Monitor Connection (Audio Engine Not Ready?)")
                    .height(Stretch(1.0))
                    .background_color(Color::rgb(20, 20, 20));
            }

            // Controls Area
            HStack::new(cx, |cx| {
                // Setup sliders
                let make_slider = |cx: &mut Context, label: &str| {
                    VStack::new(cx, |cx| {
                        Label::new(cx, label).font_size(12.0).color(Color::white());
                        // ParamSlider::new(cx, Data::new(params.as_ref()), lens);
                        Label::new(cx, "Slider Placeholder");
                    })
                    .width(Stretch(1.0))
                    .child_left(Stretch(1.0))
                    .child_right(Stretch(1.0)); // Center content
                };

                make_slider(cx, "Low");
                make_slider(cx, "L-Mid");
                make_slider(cx, "Mid");
                make_slider(cx, "H-Mid");
                make_slider(cx, "High");
            })
            .height(Pixels(100.0))
            .col_between(Pixels(10.0))
            .child_top(Pixels(10.0))
            .child_bottom(Pixels(10.0));
        })
        .row_between(Pixels(10.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));
    })
}
