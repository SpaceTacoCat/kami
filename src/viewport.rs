use crate::{Color, PresentMode, SurfaceConfiguration, TextureUsages};
use anyhow::Context;
use wgpu::{Adapter, Device, Instance, Surface, SurfaceTexture};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct ViewportDescriptor {
    pub window: Window,
    pub surface: Surface,
    pub bg: Color,
}

pub struct Viewport {
    pub descriptor: ViewportDescriptor,
    pub config: SurfaceConfiguration,
}

impl ViewportDescriptor {
    pub fn new(window: Window, bg: Color, instance: &Instance) -> Self {
        let surface = unsafe { instance.create_surface(&window) };

        Self {
            window,
            surface,
            bg,
        }
    }

    pub fn build(self, adapter: &Adapter, device: &Device) -> anyhow::Result<Viewport> {
        let window_size = self.window.inner_size();

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self
                .surface
                .get_preferred_format(adapter)
                .context("Unable to retrieve preferred format")?,
            width: window_size.width,
            height: window_size.height,
            present_mode: PresentMode::Fifo,
        };

        self.surface.configure(device, &config);

        Ok(Viewport {
            descriptor: self,
            config,
        })
    }
}

impl Viewport {
    pub fn resize(&mut self, device: &Device, new_size: PhysicalSize<u32>) {
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.descriptor.surface.configure(device, &self.config);
    }

    pub fn current_texture(&mut self) -> anyhow::Result<SurfaceTexture> {
        Ok(self.descriptor.surface.get_current_texture()?)
    }
}
