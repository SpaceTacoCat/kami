#![feature(never_type)]
#![feature(once_cell)]
#![feature(default_free_fn)]

use crate::buffer::text_buffer::{AppendOnlyTextBuffer, FontConfig};
use crate::buffer::BoundingBox;
use crate::events::KamiEvent;
use crate::render::RenderEvent;
use crate::state::{AppState, StateEvent};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{mpsc, RwLock};
use viewport::{Viewport, ViewportDescriptor};
use wgpu::{Color, PresentMode, SurfaceConfiguration, TextureUsages};
use wgpu_glyph::Section;
use winit::event::{Event, KeyboardInput, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

mod buffer;
mod events;
mod layout;
mod render;
mod state;
mod viewport;

type SharedState = Arc<RwLock<AppState>>;

pub struct WindowData {
    viewport: Viewport,
    state: SharedState,
}

pub async fn run(event_loop: EventLoop<KamiEvent>, window: Window) -> anyhow::Result<!> {
    let (render_tx, render_rx) = mpsc::channel(1024);
    let (state_tx, state_rx) = mpsc::channel(1024);

    tokio::spawn(state::state_loop(event_loop.create_proxy(), state_rx));
    tokio::spawn(async {
        render::render_loop(window, render_rx)
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
                let state_tx = state_tx.clone();
                let time_now = SystemTime::now();

                handle.spawn(async move {
                    state_tx
                        .send(StateEvent::KeyEvent(state, virtual_keycode, time_now))
                        .await
                        .unwrap()
                });
            }
            _ => {}
        }
    });
}
