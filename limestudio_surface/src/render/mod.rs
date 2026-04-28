pub mod sdf;
pub mod node_renderer;
pub mod cable_renderer;
pub mod lens_renderer;
pub mod text_renderer;
pub mod waveform_renderer;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniforms {
    pub view_proj: glam::Mat4,
    pub time: f32,
    pub _pad: [f32; 3],
}

pub struct SurfaceRenderer {
    pub typography: text_renderer::TypographySystem,
    pub sdf: sdf::SdfPipeline,
    pub cable: cable_renderer::CableRenderer,
    pub waveform: waveform_renderer::WaveformRenderer,
    pub global_uniform_buffer: wgpu::Buffer,
    pub global_bind_group: wgpu::BindGroup,
}

impl SurfaceRenderer {
    pub async fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let typography = text_renderer::TypographySystem::new(device, _queue, format);
        let sdf = sdf::SdfPipeline::new(device, format);
        let cable = cable_renderer::CableRenderer::new(device, format);
        let waveform = waveform_renderer::WaveformRenderer::new(device, format);

        let global_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Global Uniform Buffer"),
            size: std::mem::size_of::<GlobalUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Global Bind Group"),
            layout: &cable.global_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: global_uniform_buffer.as_entire_binding(),
                },
            ],
        });


        Self {
            typography,
            sdf,
            cable,
            waveform,
            global_uniform_buffer,
            global_bind_group,
        }
    }

    pub fn render_scene(
        &mut self, 
        device: &wgpu::Device, 
        queue: &wgpu::Queue, 
        view: &wgpu::TextureView, 
        time: f32,
        view_proj: glam::Mat4,
        instances: &[sdf::SdfInstance],
        primitives: &[crate::ui_ir::SurfacePrimitive],
    ) {

        // 1. Update Uniforms
        let uniforms = GlobalUniforms {
            view_proj,
            time,
            _pad: [0.0; 3],
        };
        queue.write_buffer(&self.global_uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        // 2. Collect and update Cable instances from primitives
        let mut active_cables = Vec::new();
        for prim in primitives {
            if let crate::ui_ir::SurfacePrimitive::Curve { control_points, color, thickness, kind, .. } = prim {
                if control_points.len() >= 2 {
                    let start = glam::Vec2::from(control_points[0]);
                    let end = glam::Vec2::from(control_points[control_points.len() - 1]);
                    
                    // Simple logic for automatic control points if not provided
                    let (cp1, cp2) = if control_points.len() >= 4 {
                        (glam::Vec2::from(control_points[1]), glam::Vec2::from(control_points[2]))
                    } else {
                        // Horizontal "S" curve logic for node cables
                        let dx = (end.x - start.x).abs().max(40.0);
                        (start + glam::vec2(dx * 0.5, 0.0), end - glam::vec2(dx * 0.5, 0.0))
                    };

                    let (speed, intensity) = match kind {
                        crate::ui_ir::CurveKind::Flow { direction: _, phase: _, density: _ } => (2.0, 0.5),
                        _ => (0.0, 0.0),
                    };

                    active_cables.push(cable_renderer::CableInstance {
                        start,
                        end,
                        cp1,
                        cp2,
                        color: glam::Vec4::from(*color),
                        thickness: *thickness,
                        intensity,
                        speed,
                        phase: 0.0,
                    });
                }
            }
        }

        self.sdf.write_instances(queue, instances);
        if !active_cables.is_empty() {
            queue.write_buffer(&self.cable.instance_buffer, 0, bytemuck::cast_slice(&active_cables));
        }

        // 3. Update Text
        self.typography.clear();
        for prim in primitives {
            if let crate::ui_ir::SurfacePrimitive::Text { rect, text, font_size, color, .. } = prim {
                self.typography.add_text(
                    text, 
                    glam::Vec2::new(rect[0], rect[1]), 
                    *font_size, 
                    *font_size * 1.2, 
                    crate::color::Color::from_rgba_f32(color[0], color[1], color[2], color[3])
                );
            }
        }

        // 4. Begin render pass
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Surface Render Encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surface Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.03,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Draw SDF shapes (Nodes, Knobs, Rings)
            self.sdf.draw(&mut rpass, &self.global_bind_group, instances.len() as u32);

            // Draw Cables
            if !active_cables.is_empty() {
                self.cable.draw(&mut rpass, &self.global_bind_group, &active_cables);
            }


            // Draw Typography
            self.typography.render(device, queue, &mut rpass);
        }

        // 5. Submit
        queue.submit(std::iter::once(encoder.finish()));
    }
}

