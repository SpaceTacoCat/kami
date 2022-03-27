use crate::buffer::Buffer;
use crate::layout::Layout;
use crate::KamiEvent;
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::sync::mpsc::Receiver;
use tracing::info;
use winit::event::{ElementState, VirtualKeyCode};
use winit::event_loop::EventLoopProxy;

#[derive(Default)]
pub struct AppState {
    pub buffers: Vec<Box<dyn Buffer + Send + Sync + 'static>>,
    layout: Layout,
    pub keyboard_state: KeyboardState,
}

#[derive(Default)]
pub struct KeyboardState {
    pub keypress_state: KeypressState,
    change: Vec<KeyChange>,
}

#[derive(Default, PartialEq, Eq)]
pub struct KeypressState {
    pub keys_pressed: HashMap<VirtualKeyCode, KeyPress>,
}

#[derive(PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct KeyPress {
    pub pressed_since: SystemTime,
}

pub struct KeyChange {
    pub pressed: bool,
    pub key_code: VirtualKeyCode,
}

impl KeyboardState {
    pub fn key_press(&mut self, key_code: VirtualKeyCode) {
        let time_now = SystemTime::now();
        self.keypress_state.keys_pressed.insert(
            key_code,
            KeyPress {
                pressed_since: time_now,
            },
        );

        self.change.push(KeyChange {
            pressed: true,
            key_code,
        });
    }

    pub fn key_release(&mut self, key_code: VirtualKeyCode) {
        self.keypress_state.keys_pressed.remove(&key_code);

        self.change.push(KeyChange {
            pressed: false,
            key_code,
        });
    }
}

#[derive(Debug)]
pub enum StateEvent {
    KeyEvent(ElementState, Option<VirtualKeyCode>, SystemTime),
}

pub async fn state_loop(_proxy: EventLoopProxy<KamiEvent>, mut state_rx: Receiver<StateEvent>) {
    loop {
        let event = state_rx.recv().await.unwrap();

        info!("{:?}", event);
    }
}
