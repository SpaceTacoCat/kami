mod cursor_brush;

use crate::buffer::dummy_buffer::cursor_brush::{Cursor, CursorBrush};
use crate::buffer::{BoundingBox, Buffer, BufferEvent, EventHandlerOutcome};
use crate::{Section, WindowData};
use anyhow::Context;
use core::slice;
use unicode_segmentation::UnicodeSegmentation;
use wgpu::util::StagingBelt;
use wgpu::{Adapter, CommandEncoder, Device, TextureView};
use wgpu_glyph::ab_glyph::{Font, FontArc, ScaleFont};
use wgpu_glyph::{
    BuiltInLineBreaker, GlyphBrush, GlyphBrushBuilder, GlyphCruncher, HorizontalAlign, Layout,
    Text, VerticalAlign,
};

const BACKSPACE_CHAR: char = '\u{08}';

pub struct DummyBuffer {
    text: String,
    config: FontConfig,
    cursor: usize,

    // Init
    glyph_brush: Option<GlyphBrush<()>>,
    cursor_brush: Option<CursorBrush>,
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
            cursor_brush: None,
        }
    }

    fn glyph_brush(&mut self) -> &mut GlyphBrush<()> {
        self.glyph_brush.as_mut().expect("buffer not initialized")
    }

    fn cursor_brush(&mut self) -> &mut CursorBrush {
        self.cursor_brush
            .as_mut()
            .expect("pipeline not initialized")
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

        self.cursor_brush = Some(CursorBrush::new(device, render_format))
    }

    fn enqueue(&mut self, bb: BoundingBox) {
        let text = self.text.clone();
        let color = self.config.color;
        let scale = self.config.scale;

        let section = Section {
            screen_position: (bb.left, bb.top),
            bounds: (bb.width, bb.height),
            text: vec![Text::new(&text).with_color(color).with_scale(scale)],
            layout: Layout::Wrap {
                line_breaker: BuiltInLineBreaker::UnicodeLineBreaker,
                h_align: HorizontalAlign::Left,
                v_align: VerticalAlign::Top,
            },
        };

        // Draw text
        self.glyph_brush().queue(section.clone());

        // Draw cursor
        let scaled_font = self.config.font.as_scaled(self.config.scale);
        let height = scaled_font.height();

        let (x, y) = if text.is_empty() {
            (0.0, 0.0)
        } else if text.ends_with('\n') {
            (0.0, text.lines().count() as f32 * height)
        } else {
            let glyphs = self.glyph_brush().glyphs(section);

            let pos = glyphs.last().unwrap().glyph.position;
            (pos.x, pos.y)
        };

        let mut color = [0.0f32; 3];
        color.copy_from_slice(&self.config.color[0..3]);

        self.cursor_brush().update(Cursor {
            aabb: dbg!([x, y, x + scale * 0.1, y + height]),
            z_pos: 0.0,
            color,
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
            .expect(".draw_queued can't return Err(_)");

        self.cursor_brush()
            .draw(encoder, view, device, staging_belt);
    }

    fn handle_events(&mut self, event: BufferEvent) -> EventHandlerOutcome {
        match event {
            BufferEvent::Input(c) => self.handle_input(c),
        }
    }
}
