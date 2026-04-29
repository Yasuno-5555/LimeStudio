use eframe::egui;
use eframe::egui_wgpu;
use eframe::egui_wgpu::wgpu;
use limestudio_surface::render::sdf::{SdfInstance, SdfPipeline};

pub struct CanvasRenderer {
    pub pipeline: Option<SdfPipeline>,
}

pub struct TrustCanvasCallback {
    pub sdf_instances: Vec<SdfInstance>,
}

unsafe impl Send for TrustCanvasCallback {}
unsafe impl Sync for TrustCanvasCallback {}

impl Default for CanvasRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasRenderer {
    pub fn new() -> Self {
        Self { pipeline: None }
    }

    pub fn prepare(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        if self.pipeline.is_none() {
            let p = limestudio_surface::render::sdf::SdfPipeline::new(device, format);
            self.pipeline = Some(p);
        }
    }

    pub fn render(
        &self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        raw_instances: Vec<SdfInstance>,
    ) -> egui::Response {
        let (response, painter) = ui.allocate_painter(rect.size(), egui::Sense::drag());

        // Custom egui-wgpu callback for the "Trust Canvas"
        let callback = egui_wgpu::Callback::new_paint_callback(
            rect,
            TrustCanvasCallback {
                sdf_instances: raw_instances,
            },
        );

        painter.add(callback);
        response
    }
}

impl egui_wgpu::CallbackTrait for TrustCanvasCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        // Here we would write to the instance buffer for the SDF renderer
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        _render_pass: &mut wgpu::RenderPass<'_>,
        _callback_resources: &egui_wgpu::CallbackResources,
    ) {
        // High-fidelity rendering of nodes and cables using SDFs
        // This is where the 'Visual Excellence' and 'Trust' lives.
    }
}
