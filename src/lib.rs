#![feature(never_type)]
#![feature(once_cell)]
#![feature(default_free_fn)]
#![feature(let_else)]

use crate::app_state::{AppState, SharedState};
use crate::buffer::dummy_buffer::{DummyBuffer, FontConfig};
use crate::buffer::{BoundingBox, Buffer};
use crate::events::KamiEvent;
use crate::layout::Layout;
use crate::render::RenderEvent;
use crate::state::StateEvent;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use viewport::{Viewport, ViewportDescriptor};
use wgpu::{Color, PresentMode, SurfaceConfiguration, TextureUsages};
use wgpu_glyph::ab_glyph::FontArc;
use wgpu_glyph::Section;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

mod app_state;
mod buffer;
mod events;
mod layout;
mod render;
mod state;
mod viewport;

pub struct WindowData {
    viewport: Viewport,
    state: SharedState,
}

pub async fn run(event_loop: EventLoop<KamiEvent>, window: Window) -> anyhow::Result<!> {
    let (render_tx, render_rx) = mpsc::channel(1024);
    let (state_tx, state_rx) = mpsc::channel(1024);

    let font = FontArc::try_from_slice(include_bytes!("../resources/FiraCode-Regular.ttf"))?;

    let mut app_state = AppState::default();

    // TODO: Make it init just basic buffer
    app_state
        .buffers
        .push(Arc::new(Mutex::new(DummyBuffer::new(FontConfig {
            scale: 20.0,
            color: [0.0, 0.0, 0.0, 1.0],
            font: font.clone(),
        }))));

    app_state
        .buffers
        .push(Arc::new(Mutex::new(DummyBuffer::new(FontConfig {
            scale: 40.0,
            color: [0.0, 0.0, 0.0, 1.0],
            font,
        }))));

    app_state.active_buffer = 1;

    let state = Arc::new(RwLock::new(app_state));

    tokio::spawn(state::state_loop(
        event_loop.create_proxy(),
        state_rx,
        state.clone(),
    ));
    tokio::spawn(async {
        render::render_loop(window, render_rx, state)
            .await
            .expect("Render loop failed")
    });

    event_loop.run(move |event, _, control_flow| {
        let handle = tokio::runtime::Handle::current();

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                let render_tx = render_tx.clone();
                handle.spawn(async move {
                    render_tx.send(RenderEvent::Resize(new_size)).await.unwrap()
                });
            }
            Event::RedrawRequested(_) => {
                let render_tx = render_tx.clone();
                handle.spawn(async move { render_tx.send(RenderEvent::Redraw).await.unwrap() });
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                // Handle text input
                event: WindowEvent::ReceivedCharacter(c),
                ..
            } => {
                let state_tx = state_tx.clone();

                handle.spawn(async move { state_tx.send(StateEvent::CharInput(c)).await.unwrap() });
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(modifiers),
                ..
            } => {
                let state_tx = state_tx.clone();

                handle.spawn(async move {
                    state_tx
                        .send(StateEvent::ModifiersChange(modifiers))
                        .await
                        .unwrap()
                });
            }
            Event::UserEvent(event) => match event {
                KamiEvent::RequestRedraw => {
                    let render_tx = render_tx.clone();
                    handle.spawn(async move { render_tx.send(RenderEvent::Redraw).await.unwrap() });
                }
            },
            _ => {}
        }
    });
}
