use kami::run;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let ev_loop = EventLoop::with_user_event();
    let window = WindowBuilder::new().with_title("紙").build(&ev_loop)?;

    run(ev_loop, window).await?;
}
