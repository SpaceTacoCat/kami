use crate::buffer::{BufferEvent, EventHandlerOutcome};
use crate::{Buffer, Layout};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub type SharedState = Arc<RwLock<AppState>>;

#[derive(Default)]
pub struct AppState {
    pub buffers: Vec<Arc<Mutex<dyn Buffer + Send + Sync + 'static>>>,
    pub active_buffer: usize,
    layout: Layout,
}

impl AppState {
    pub async fn handle_character(&self, c: char) -> EventHandlerOutcome {
        let mutex = self
            .buffers
            .get(self.active_buffer)
            .expect("get active buffer");

        let mut buffer = mutex.lock().await;

        buffer.handle_events(BufferEvent::Input(c))
    }
}
