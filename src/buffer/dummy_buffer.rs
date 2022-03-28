use crate::buffer::{BoundingBox, Buffer, BufferEvent, EventHandlerOutcome};
use crate::{Section, WindowData};
use anyhow::Context;
use unicode_segmentation::UnicodeSegmentation;
use wgpu::util::StagingBelt;
use wgpu::{Adapter, CommandEncoder, Device, TextureView};
use wgpu_glyph::ab_glyph::FontArc;
use wgpu_glyph::{
    BuiltInLineBreaker, GlyphBrush, GlyphBrushBuilder, HorizontalAlign, Layout, Text, VerticalAlign,
};

const BACKSPACE_CHAR: char = '\u{08}';

pub struct DummyBuffer {
    text: String,
    config: FontConfig,
    cursor: usize,

    // Init
    glyph_brush: Option<GlyphBrush<()>>,
}

pub struct FontConfig {
    pub scale: f32,
    pub color: [f32; 4],
    pub font: FontArc,
}

impl DummyBuffer {
    pub fn new(config: FontConfig) -> Self {
        Self {
            text: String::new(),
            config,
            cursor: 0,
            glyph_brush: None,
        }
    }

    fn glyph_brush(&mut self) -> &mut GlyphBrush<()> {
        self.glyph_brush.as_mut().expect("buffer not initialized")
    }

    fn handle_input(&mut self, c: char) -> EventHandlerOutcome {
        if c == BACKSPACE_CHAR && !self.text.is_empty() {
            let Some((idx, _)) = self.text.grapheme_indices(true).last() else {
                return EventHandlerOutcome::None;
            };

            self.text = self.text[..idx].to_string();
        } else {
            self.text.push(c);
        }
        EventHandlerOutcome::Redraw
    }
}

impl Buffer for DummyBuffer {
    fn init_rendering(&mut self, window_data: &WindowData, device: &Device, adapter: &Adapter) {
        let render_format = window_data
            .viewport
            .descriptor
            .surface
            .get_preferred_format(adapter)
            .context("Can't retrieve preferred surface texture format")
            .unwrap();

        self.glyph_brush = Some(
            GlyphBrushBuilder::using_font(self.config.font.clone()).build(device, render_format),
        );
    }

    fn enqueue(&mut self, bb: BoundingBox) {
        let text = self.text.clone();
        let color = self.config.color;
        let scale = self.config.scale;
        self.glyph_brush().queue(Section {
            screen_position: (bb.top, bb.left),
            bounds: (bb.width, bb.height),
            text: vec![Text::new(&text).with_color(color).with_scale(scale)],
            layout: Layout::Wrap {
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
        self.glyph_brush()
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

    fn handle_events(&mut self, event: BufferEvent) -> EventHandlerOutcome {
        match event {
            BufferEvent::Input(c) => self.handle_input(c),
        }
    }
}
