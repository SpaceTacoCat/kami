use wgpu::util::StagingBelt;
use wgpu::{CommandEncoder, Device, TextureView};

pub mod text_buffer;

pub struct BoundingBox {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

pub enum EventHandlerOutcome {
    Redraw,
    None,
}

pub enum Event {
    KeyPress(char),
}

pub trait Buffer {
    fn enqueue(&mut self, bb: BoundingBox);
    fn draw_queued(
        &mut self,
        device: &Device,
        staging_belt: &mut StagingBelt,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        target_width: u32,
        target_height: u32,
    );
    fn handle_events(&mut self, event: Event) -> EventHandlerOutcome;
}
