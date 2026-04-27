use wgpu::*;
use glam::{Vec2, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WaveformPoint {
    pub position: Vec2,
    pub _pad: [f32; 2],
    pub color: Vec4,
}

pub struct WaveformRenderer {
    pub pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,
    pub max_points: usize,
}

impl WaveformRenderer {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Waveform Shader"),
            source: ShaderSource::Wgsl(include_str!("waveform.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Waveform Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Waveform Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[
                    VertexBufferLayout {
                        array_stride: std::mem::size_of::<WaveformPoint>() as BufferAddress,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &vertex_attr_array![
                            0 => Float32x2, // position
                            1 => Float32x4, // color
                        ],
                    },
                ],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let max_points = 1024;
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Waveform Vertex Buffer"),
            size: (std::mem::size_of::<WaveformPoint>() as u64) * max_points as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            max_points,
        }
    }

    pub fn draw<'a>(&'a self, rpass: &mut RenderPass<'a>, points: &[WaveformPoint]) {
        if points.is_empty() { return; }
        rpass.set_pipeline(&self.pipeline);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.draw(0..points.len() as u32, 0..1);
    }
}
