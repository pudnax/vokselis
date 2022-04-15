use color_eyre::eyre::Result;
use pollster::FutureExt;
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

use vokselis::run;

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let event_loop = EventLoop::with_user_event();
    let window = WindowBuilder::new()
        .with_title("Vokselis")
        .with_inner_size(LogicalSize::new(1280, 720))
        .build(&event_loop)?;

    run(event_loop, window).block_on()
}
