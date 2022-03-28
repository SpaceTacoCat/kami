use crate::app_state::SharedState;
use crate::{BoundingBox, ViewportDescriptor, WindowData};
use anyhow::Context;
use std::default::default;
use tokio::sync::mpsc::Receiver;
use wgpu::util::StagingBelt;
use wgpu::{
    Backends, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance,
    Limits, LoadOp, Operations, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, TextureViewDescriptor,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

#[derive(Debug)]
pub enum RenderEvent {
    Resize(PhysicalSize<u32>),
    Redraw,
}

pub async fn render_loop(
    window: Window,
    mut rx: Receiver<RenderEvent>,
    state: SharedState,
) -> anyhow::Result<()> {
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

    let mut window_data = WindowData {
        viewport: viewport_desc
            .build(&adapter, &device)
            .expect("Build viewport"),
        state: state.clone(),
    };

    let mut staging_belt = StagingBelt::new(1024);

    for buffer in state.read().await.buffers.iter() {
        let mut buffer = buffer.lock().await;
        buffer.init_rendering(&window_data, &device, &adapter)
    }

    while let Some(event) = rx.recv().await {
        match event {
            RenderEvent::Resize(new_size) => resize_window(&mut window_data, &device, new_size),
            RenderEvent::Redraw => {
                redraw_window(&mut window_data, &device, &queue, &mut staging_belt).await;
            }
        }
    }

    Ok(())
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
    let buffers = {
        let app_state = state.read().await;
        app_state.buffers.clone()
    };

    for buffer in &buffers {
        let mut buffer = buffer.lock().await;
        buffer.enqueue(BoundingBox {
            left: 0.0,
            top: 0.0,
            width: size.width as f32,
            height: size.height as f32,
        });
    }

    for buffer in &buffers {
        let mut buffer = buffer.lock().await;
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
