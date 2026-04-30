use crate::ui_ir::{BespokeBrush, OrganicKind, PathSegment, SurfacePrimitive};
use vello::wgpu::{Device, Queue, TextureView};
use vello::{Renderer, RendererOptions, Scene};

pub struct VelloRenderer {
    pub renderer: Renderer,
    pub scene: Scene,
}

impl VelloRenderer {
    pub fn new(device: &Device) -> Self {
        let renderer = Renderer::new(
            device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: std::num::NonZeroUsize::new(1),
                pipeline_cache: None,
            },
        )
        .unwrap();

        Self {
            renderer,
            scene: Scene::new(),
        }
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        view: &TextureView,
        width: u32,
        height: u32,
        primitives: &[SurfacePrimitive],
        signal_registry: &std::collections::HashMap<
            crate::ui_ir::SignalId,
            std::sync::Arc<dyn std::any::Any + Send + Sync>,
        >,
        history_store: &std::collections::HashMap<
            crate::ui_ir::SurfaceId,
            std::collections::VecDeque<SurfacePrimitive>,
        >,
    ) {
        self.scene.reset();

        for prim in primitives {
            self.draw_primitive(prim, signal_registry, history_store);
        }

        self.renderer
            .render_to_texture(
                device,
                queue,
                &self.scene,
                view,
                &vello::RenderParams {
                    base_color: vello::peniko::Color::from_rgba8(10, 10, 15, 255),
                    width,
                    height,
                    antialiasing_method: vello::AaConfig::Area,
                },
            )
            .unwrap();
    }

    fn draw_primitive(
        &mut self,
        prim: &SurfacePrimitive,
        signal_registry: &std::collections::HashMap<
            crate::ui_ir::SignalId,
            std::sync::Arc<dyn std::any::Any + Send + Sync>,
        >,
        history_store: &std::collections::HashMap<
            crate::ui_ir::SurfaceId,
            std::collections::VecDeque<SurfacePrimitive>,
        >,
    ) {
        use vello::kurbo::{Affine, BezPath, Circle, Rect, RoundedRect, Stroke};
        use vello::peniko::{Brush, Color, Fill};

        match prim {
            SurfacePrimitive::Frame {
                rect, color, style, ..
            } => {
                let r = Rect::new(
                    rect[0] as f64,
                    rect[1] as f64,
                    (rect[0] + rect[2]) as f64,
                    (rect[1] + rect[3]) as f64,
                );
                let c = Color::from_rgba8(
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                    (color[3] * 255.0) as u8,
                );

                let radius = match style {
                    crate::ui_ir::FrameStyle::Standard => 8.0,
                    _ => 0.0,
                };

                if radius > 0.0 {
                    let rounded = RoundedRect::from_rect(r, radius);
                    self.scene
                        .fill(Fill::NonZero, Affine::IDENTITY, c, None, &rounded);
                } else {
                    self.scene
                        .fill(Fill::NonZero, Affine::IDENTITY, c, None, &r);
                }
            }
            SurfacePrimitive::Organic { kind, brush, .. } => {
                let vello_brush = self.convert_brush(brush);
                match kind {
                    OrganicKind::Aura {
                        center,
                        radius,
                        pulsation,
                        ..
                    } => {
                        let c = Circle::new(
                            (center[0] as f64, center[1] as f64),
                            (*radius as f64) * (1.0 + *pulsation as f64 * 0.1),
                        );
                        self.scene
                            .fill(Fill::NonZero, Affine::IDENTITY, &vello_brush, None, &c);
                    }
                    OrganicKind::LiveStream { id, thickness, .. } => {
                        if let Some(signal) = signal_registry.get(id) {
                            if let Some(buffer) =
                                signal.downcast_ref::<crate::runtime::signal::SignalBuffer>()
                            {
                                let samples = buffer.read_all();
                                if samples.len() > 1 {
                                    let mut path = BezPath::new();
                                    // Hack: Render as a simple horizontal waveform for now
                                    let step = 100.0 / (samples.len() - 1) as f64;
                                    path.move_to((0.0, samples[0] as f64 * 50.0));
                                    for (i, &s) in samples.iter().enumerate().skip(1) {
                                        path.line_to((i as f64 * step, s as f64 * 50.0));
                                    }
                                    let stroke = Stroke::new(*thickness as f64);
                                    self.scene.stroke(
                                        &stroke,
                                        Affine::IDENTITY,
                                        &vello_brush,
                                        None,
                                        &path,
                                    );
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            SurfacePrimitive::Path {
                segments,
                brush,
                stroke,
                ..
            } => {
                let vello_brush = self.convert_brush(brush);
                let mut path = BezPath::new();
                for seg in segments {
                    match seg {
                        PathSegment::MoveTo(p) => path.move_to((p[0] as f64, p[1] as f64)),
                        PathSegment::LineTo(p) => path.line_to((p[0] as f64, p[1] as f64)),
                        PathSegment::QuadTo(p1, p2) => {
                            path.quad_to((p1[0] as f64, p1[1] as f64), (p2[0] as f64, p2[1] as f64))
                        }
                        PathSegment::CurveTo(p1, p2, p3) => path.curve_to(
                            (p1[0] as f64, p1[1] as f64),
                            (p2[0] as f64, p2[1] as f64),
                            (p3[0] as f64, p3[1] as f64),
                        ),
                        PathSegment::Close => path.close_path(),
                    }
                }

                if let Some(s) = stroke {
                    let vello_stroke = vello::kurbo::Stroke::new(s.thickness as f64);
                    self.scene
                        .stroke(&vello_stroke, Affine::IDENTITY, &vello_brush, None, &path);
                } else {
                    self.scene
                        .fill(Fill::NonZero, Affine::IDENTITY, &vello_brush, None, &path);
                }
            }
            SurfacePrimitive::PersistenceTrail {
                source_id, depth, ..
            } => {
                if let Some(history) = history_store.get(source_id) {
                    for past_prim in history.iter().take(*depth) {
                        self.draw_primitive(past_prim, signal_registry, history_store);
                    }
                }
            }
            _ => {}
        }
    }

    fn convert_brush(&self, brush: &BespokeBrush) -> vello::peniko::Brush {
        use vello::peniko::{Brush, Color, Gradient};
        match brush {
            BespokeBrush::Solid(c) => Brush::Solid(Color::from_rgba8(
                (c[0] * 255.0) as u8,
                (c[1] * 255.0) as u8,
                (c[2] * 255.0) as u8,
                (c[3] * 255.0) as u8,
            )),
            _ => Brush::Solid(Color::WHITE), // Placeholder for complex gradients
        }
    }
}
