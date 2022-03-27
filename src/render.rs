use crate::{
    AppState, AppendOnlyTextBuffer, BoundingBox, FontConfig, ViewportDescriptor, WindowData,
};
use anyhow::Context;
use std::default::default;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;
use wgpu::util::StagingBelt;
use wgpu::{
    Backends, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance,
    Limits, LoadOp, Operations, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, TextureViewDescriptor,
};
use wgpu_glyph::ab_glyph::FontArc;
use wgpu_glyph::GlyphBrushBuilder;
use winit::dpi::PhysicalSize;
use winit::window::Window;

#[derive(Debug)]
pub enum RenderEvent {
    Resize(PhysicalSize<u32>),
    Redraw,
}

pub async fn render_loop(window: Window, mut rx: Receiver<RenderEvent>) -> anyhow::Result<()> {
    let instance = Instance::new(Backends::all());
    let viewport_desc = ViewportDescriptor::new(
        window,
        Color {
            r: 0.4,
            g: 0.4,
            b: 0.4,
            a: 1.0,
        },
        &instance,
    );
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&viewport_desc.surface),
            ..default()
        })
        .await
        .context("Failed to find an appropriate adapter")?;

    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: None,
                features: Features::empty(),
                limits: Limits::downlevel_defaults(),
            },
            None,
        )
        .await
        .context("Failed to create device")?;

    let font = FontArc::try_from_slice(include_bytes!("../resources/FiraCode-Regular.ttf"))?;

    let render_format = viewport_desc
        .surface
        .get_preferred_format(&adapter)
        .context("Can't retrieve preferred surface texture format")?;

    let glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, render_format);

    let mut app_state = AppState::default();

    app_state.buffers.push(Box::new(AppendOnlyTextBuffer::new(
        FontConfig {
            scale: 20.0,
            color: [0.0, 0.0, 0.0, 1.0],
        },
        glyph_brush,
    )));

    let state = Arc::new(RwLock::new(app_state));

    let mut window_data = WindowData {
        viewport: viewport_desc
            .build(&adapter, &device)
            .expect("Build viewport"),
        state: state.clone(),
    };

    let mut staging_belt = StagingBelt::new(1024);

    loop {
        let event = rx.recv().await.unwrap();

        match event {
            RenderEvent::Resize(new_size) => resize_window(&mut window_data, &device, new_size),
            RenderEvent::Redraw => {
                redraw_window(&mut window_data, &device, &queue, &mut staging_belt).await;
            }
        }
    }
}

fn resize_window(window_data: &mut WindowData, device: &Device, new_size: PhysicalSize<u32>) {
    window_data.viewport.resize(device, new_size);
    window_data.viewport.descriptor.window.request_redraw();
}

async fn redraw_window(
    window_data: &mut WindowData,
    device: &Device,
    queue: &Queue,
    staging_belt: &mut StagingBelt,
) {
    let frame = window_data
        .viewport
        .current_texture()
        .expect("Couldn't fetch current texture");
    let view = frame.texture.create_view(&TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    let _ = encoder.begin_render_pass(&RenderPassDescriptor {
        label: None,
        color_attachments: &[RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(window_data.viewport.descriptor.bg),
                store: true,
            },
        }],
        depth_stencil_attachment: None,
    });

    let size = window_data.viewport.descriptor.window.inner_size();

    let state = window_data.state.clone();
    let mut app_state = state.write().await;
    for buffer in app_state.buffers.iter_mut() {
        buffer.enqueue(BoundingBox {
            left: 0.0,
            top: 0.0,
            width: size.width as f32,
            height: size.height as f32,
        });
    }

    for buffer in app_state.buffers.iter_mut() {
        buffer.draw_queued(
            device,
            staging_belt,
            &mut encoder,
            &view,
            size.width,
            size.height,
        );
    }

    staging_belt.finish();
    queue.submit(Some(encoder.finish()));
    frame.present();

    tokio::spawn(staging_belt.recall());
}
