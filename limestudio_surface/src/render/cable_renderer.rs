use wgpu::*;
use glam::{Vec2, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CableInstance {
    pub start: Vec2,
    pub end: Vec2,
    pub color: Vec4,
    pub thickness: f32,
    pub intensity: f32,
    pub speed: f32,
    pub phase: f32,
}

pub struct CableRenderer {
    pub pipeline: RenderPipeline,
    pub instance_buffer: Buffer,
    pub global_bind_group_layout: BindGroupLayout,
    pub max_instances: usize,
}

impl CableRenderer {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_wgsl!("cable.wgsl"));

        let global_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Cable Global Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Cable Renderer Layout"),
            bind_group_layouts: &[&global_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Cable Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[
                    VertexBufferLayout {
                        array_stride: std::mem::size_of::<CableInstance>() as BufferAddress,
                        step_mode: VertexStepMode::Instance,
                        attributes: &vertex_attr_array![
                            0 => Float32x2, // start
                            1 => Float32x2, // end
                            2 => Float32x4, // color
                            3 => Float32,   // thickness
                            4 => Float32,   // intensity
                            5 => Float32,   // speed
                            6 => Float32,   // phase
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
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let max_instances = 1024;
        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Cable Instance Buffer"),
            size: (std::mem::size_of::<CableInstance>() * max_instances) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            instance_buffer,
            global_bind_group_layout,
            max_instances,
        }
    }

    pub fn draw<'a>(&'a self, rpass: &mut RenderPass<'a>, global_bind_group: &'a BindGroup, instances: &[CableInstance]) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, global_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        rpass.draw(0..4, 0..instances.len() as u32);
    }
}
