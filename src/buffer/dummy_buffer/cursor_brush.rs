use std::borrow::Cow;
use std::default::default;
use std::mem::size_of;
use std::num::NonZeroU64;
use std::slice::from_raw_parts;
use wgpu::util::StagingBelt;
use wgpu::{
    vertex_attr_array, BlendComponent, BlendFactor, BlendOperation, BlendState, Buffer,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, CommandEncoder, Device,
    FragmentState, FrontFace, IndexFormat, LoadOp, Operations, PrimitiveState, PrimitiveTopology,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderSource, TextureView, VertexBufferLayout, VertexState, VertexStepMode,
};

pub struct CursorBrush {
    raw: RenderPipeline,
    buffer: Buffer,

    cursor: Cursor,
}

#[derive(Default)]
#[repr(C)]
pub struct Cursor {
    pub aabb: [f32; 4],
    pub z_pos: f32,
    pub color: [f32; 3],
}

impl CursorBrush {
    pub fn new(device: &wgpu::Device, render_format: wgpu::TextureFormat) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("kami::pipeline buffer"),
            size: size_of::<Cursor>() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[],
        });

        let shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("./additions.wgsl"))),
        });

        let raw = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: 32,
                    step_mode: VertexStepMode::Instance,
                    attributes: &vertex_attr_array![
                        0 => Float32x4,
                        1 => Float32,
                        2 => Float32x3,
                    ],
                }],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(IndexFormat::Uint16),
                front_face: FrontFace::Cw,
                ..default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: render_format,
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                    }),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            multiview: None,
        });

        Self {
            raw,
            buffer,
            cursor: default(),
        }
    }

    pub fn update(&mut self, cursor: Cursor) {
        self.cursor = cursor;
    }

    pub fn draw(
        &mut self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        device: &Device,
        staging_belt: &mut StagingBelt,
    ) {
        let cursor_bytes = unsafe {
            // This reinterpret works when cursor is repr(C)
            from_raw_parts(
                &self.cursor as *const Cursor as *const u8,
                size_of::<Cursor>(),
            )
        };

        let mut buffer_view = staging_belt.write_buffer(
            encoder,
            &self.buffer,
            0,
            NonZeroU64::new(size_of::<Cursor>() as u64).unwrap(),
            device,
        );

        buffer_view.copy_from_slice(cursor_bytes);

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("kami::pipeline render pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.raw);
        render_pass.set_vertex_buffer(0, self.buffer.slice(..));

        render_pass.draw(0..4, 0..1);
    }
}

pub fn orthographic_projection(width: u32, height: u32) -> [f32; 16] {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    [
        2.0 / width as f32, 0.0, 0.0, 0.0,
        0.0, -2.0 / height as f32, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        -1.0, 1.0, 0.0, 1.0,
    ]
}
