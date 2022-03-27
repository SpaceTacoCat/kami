use crate::{KamiEvent, SharedState};
use std::collections::HashMap;
use std::lazy::Lazy;
use std::sync::{Arc, Condvar};
use std::time::SystemTime;
use tokio::sync::{Notify, RwLock};
use tracing::info;
use winit::event::VirtualKeyCode;
use winit::event_loop::EventLoopProxy;
use winit::window::WindowId;

#[derive(Default)]
pub struct TemporaryState {
    pub keyboard_state: KeyboardState,
    old_state: KeyboardState,
    sleep_var: Arc<Notify>,
}

#[derive(Default, PartialEq, Eq)]
pub struct KeyboardState {
    pub keys_pressed: HashMap<VirtualKeyCode, KeyPress>,
}

#[derive(PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct KeyPress {
    pub pressed_since: SystemTime,
}

impl TemporaryState {
    pub fn key_press(&mut self, key_code: VirtualKeyCode) {
        self.keyboard_state.keys_pressed.insert(
            key_code,
            KeyPress {
                pressed_since: SystemTime::now(),
            },
        );

        self.sleep_var.notify_one();
    }

    pub fn key_release(&mut self, key_code: VirtualKeyCode) {
        self.keyboard_state.keys_pressed.remove(&key_code);
    }
}

pub async fn state_loop(state: SharedState, proxy: EventLoopProxy<KamiEvent>) {
    let sleep_var = state.clone().read().await.sleep_var.clone();

    loop {
        sleep_var.notified().await;
    }
}
