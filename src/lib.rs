#![feature(never_type)]
#![feature(once_cell)]
#![feature(default_free_fn)]

use crate::events::KamiEvent;
use crate::state::TemporaryState;
use anyhow::Context;
use std::default::default;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::RwLock;
use viewport::{Viewport, ViewportDescriptor};
use wgpu::util::StagingBelt;
use wgpu::{
    Backends, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance,
    Limits, LoadOp, Operations, PresentMode, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptions, SurfaceConfiguration, TextureFormat,
    TextureUsages, TextureViewDescriptor,
};
use wgpu_glyph::ab_glyph::FontArc;
use wgpu_glyph::{
    BuiltInLineBreaker, GlyphBrush, GlyphBrushBuilder, HorizontalAlign, Layout, Section, Text,
    VerticalAlign,
};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};

mod events;
mod state;
mod viewport;

type SharedState = Arc<RwLock<TemporaryState>>;

pub struct WindowData {
    viewport: Viewport,
    state: SharedState,
}

pub async fn run(
    event_loop: EventLoop<KamiEvent>,
    window: Window,
    color: Color,
) -> anyhow::Result<!> {
    let instance = Instance::new(Backends::all());
    let viewport_desc = ViewportDescriptor::new(window, color, &instance);
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

    let state = Arc::new(RwLock::new(TemporaryState::default()));
    let mut window_data = WindowData {
        viewport: viewport_desc
            .build(&adapter, &device)
            .expect("Build viewport"),
        state: state.clone(),
    };

    tokio::spawn(state::state_loop(state, event_loop.create_proxy()));

    let mut staging_belt = StagingBelt::new(1024);

    let fira_code = FontArc::try_from_slice(include_bytes!("../resources/FiraCode-Regular.ttf"))?;
    let mut glyph_brush = GlyphBrushBuilder::using_font(fira_code).build(
        &device,
        window_data
            .viewport
            .descriptor
            .surface
            .get_preferred_format(&adapter)
            .context("Can't retrieve preferred surface texture format")?,
    );

    event_loop.run(move |event, _, control_flow| {
        let _ = (&instance, &adapter);

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => resize_window(&mut window_data, &device, new_size),
            Event::RedrawRequested(_) => {
                redraw_window(
                    &mut window_data,
                    &device,
                    &queue,
                    &mut glyph_brush,
                    &mut staging_belt,
                );
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => close_window(control_flow),
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                accept_user_keyboard_input(window_data.state.clone(), state, virtual_keycode);
            }
            Event::UserEvent(user_event) => match user_event {
                KamiEvent::RequestRedraw => window_data.viewport.descriptor.window.request_redraw(),
            },
            _ => {}
        }
    });
}

fn accept_user_keyboard_input(
    shared_state: SharedState,
    state: ElementState,
    virtual_keycode: Option<VirtualKeyCode>,
) {
    if let Some(key_code) = virtual_keycode {
        let handle = Handle::current();

        match state {
            ElementState::Pressed => {
                handle.spawn(async move { shared_state.write().await.key_press(key_code) });
            }
            ElementState::Released => {
                handle.spawn(async move { shared_state.write().await.key_release(key_code) });
            }
        }
    }
}

fn close_window(control_flow: &mut ControlFlow) {
    *control_flow = ControlFlow::Exit
}

fn resize_window(window_data: &mut WindowData, device: &Device, new_size: PhysicalSize<u32>) {
    window_data.viewport.resize(device, new_size);
    window_data.viewport.descriptor.window.request_redraw();
}

fn redraw_window(
    window_data: &mut WindowData,
    device: &Device,
    queue: &Queue,
    glyph_brush: &mut GlyphBrush<()>,
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

    glyph_brush.queue(Section {
        screen_position: (0.0, 0.0),
        bounds: (size.width as f32, size.height as f32),
        text: vec![Text::new("Lorem ipsum dolor sit amet, consectetur adipis")
            .with_color([0.0, 0.0, 0.0, 1.0])
            .with_scale(40.0)],
        layout: Layout::SingleLine {
            line_breaker: BuiltInLineBreaker::UnicodeLineBreaker,
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Top,
        },
    });

    glyph_brush
        .draw_queued(
            &device,
            staging_belt,
            &mut encoder,
            &view,
            size.width,
            size.height,
        )
        .unwrap();

    staging_belt.finish();
    queue.submit(Some(encoder.finish()));
    frame.present();

    let handle = Handle::current();

    handle.spawn(staging_belt.recall());
}
