use glam::{Vec2, Vec4};
use wgpu::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NodeInstance {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub corner_radius: f32,
    pub border_thickness: f32,
    pub _pad: [f32; 2],
}

pub struct NodeRenderer {
    pub pipeline: RenderPipeline,
    pub instance_buffer: Buffer,
    pub max_instances: usize,
}

impl NodeRenderer {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_wgsl!("node.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Node Renderer Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Node Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<NodeInstance>() as BufferAddress,
                    step_mode: VertexStepMode::Instance,
                    attributes: &vertex_attr_array![
                        0 => Float32x2, // position
                        1 => Float32x2, // size
                        2 => Float32x4, // color
                        3 => Float32,   // corner_radius
                        4 => Float32,   // border_thickness
                    ],
                }],
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
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let max_instances = 1024;
        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Node Instance Buffer"),
            size: (std::mem::size_of::<NodeInstance>() * max_instances) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            instance_buffer,
            max_instances,
        }
    }

    pub fn draw<'a>(&'a self, rpass: &mut RenderPass<'a>, instances: &[NodeInstance]) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        rpass.draw(0..4, 0..instances.len() as u32);
    }
}
