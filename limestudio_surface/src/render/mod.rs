pub mod sdf;
pub mod node_renderer;
pub mod cable_renderer;
pub mod lens_renderer;
pub mod text_renderer;
pub mod waveform_renderer;

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
            size: 16, // vec4
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
        instances: &[sdf::SdfInstance],
        cables: &[cable_renderer::CableInstance],
    ) {
        // 1. Update Uniforms
        queue.write_buffer(&self.global_uniform_buffer, 0, bytemuck::bytes_of(&time));

        // 2. Update instance buffers
        self.sdf.write_instances(queue, instances);
        queue.write_buffer(&self.cable.instance_buffer, 0, bytemuck::cast_slice(cables));

        // 3. Begin render pass
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
            rpass.set_pipeline(&self.sdf.pipeline);
            rpass.set_vertex_buffer(0, self.sdf.instance_buffer.slice(..));
            rpass.draw(0..4, 0..instances.len() as u32);

            // Draw Cables
            self.cable.draw(&mut rpass, &self.global_bind_group, cables);
        }

        // 4. Submit
        queue.submit(std::iter::once(encoder.finish()));
    }
}
