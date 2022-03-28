use crate::WindowData;
use wgpu::util::StagingBelt;
use wgpu::{Adapter, CommandEncoder, Device, TextureView};

pub mod dummy_buffer;

#[derive(Copy, Clone, Debug)]
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

pub enum BufferEvent {
    Input(char),
}

pub trait Buffer {
    fn init_rendering(&mut self, window_data: &WindowData, device: &Device, adapter: &Adapter);
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
    fn handle_events(&mut self, event: BufferEvent) -> EventHandlerOutcome;
}
