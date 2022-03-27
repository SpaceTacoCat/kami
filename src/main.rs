use kami::run;
use wgpu::Color;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let ev_loop = EventLoop::with_user_event();
    let window = WindowBuilder::new().with_title("紙").build(&ev_loop)?;

    run(
        ev_loop,
        window,
        Color {
            r: 0.4,
            g: 0.4,
            b: 0.4,
            a: 1.0,
        },
    )
    .await?;
}
