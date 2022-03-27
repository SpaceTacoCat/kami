use crate::buffer::{BoundingBox, Buffer, Event, EventHandlerOutcome};
use crate::Section;
use wgpu::util::StagingBelt;
use wgpu::{CommandEncoder, Device, TextureView};
use wgpu_glyph::{BuiltInLineBreaker, GlyphBrush, HorizontalAlign, Layout, Text, VerticalAlign};

pub struct AppendOnlyTextBuffer {
    text: String,
    config: FontConfig,
    glyph_brush: GlyphBrush<()>,
}

pub struct FontConfig {
    pub scale: f32,
    pub color: [f32; 4],
}

impl AppendOnlyTextBuffer {
    pub fn new(config: FontConfig, glyph_brush: GlyphBrush<()>) -> Self {
        Self {
            text: "placeholder text".to_string(),
            config,
            glyph_brush,
        }
    }
}

impl Buffer for AppendOnlyTextBuffer {
    fn enqueue(&mut self, bb: BoundingBox) {
        self.glyph_brush.queue(Section {
            screen_position: (bb.top, bb.left),
            bounds: (bb.width, bb.height),
            text: vec![Text::new(&self.text)
                .with_color(self.config.color)
                .with_scale(self.config.scale)],
            layout: Layout::SingleLine {
                line_breaker: BuiltInLineBreaker::UnicodeLineBreaker,
                h_align: HorizontalAlign::Left,
                v_align: VerticalAlign::Top,
            },
        });
    }

    fn draw_queued(
        &mut self,
        device: &Device,
        staging_belt: &mut StagingBelt,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        target_width: u32,
        target_height: u32,
    ) {
        self.glyph_brush
            .draw_queued(
                device,
                staging_belt,
                encoder,
                view,
                target_width,
                target_height,
            )
            .expect(".draw_queued can't return Err(_)")
    }

    fn handle_events(&mut self, event: Event) -> EventHandlerOutcome {
        match event {
            Event::KeyPress(c) => {
                self.text.push(c);
                EventHandlerOutcome::Redraw
            }
        }
    }
}
