use color_eyre::eyre::Result;
use pollster::FutureExt;
use winit::{event_loop::EventLoop, window::WindowBuilder};

use vokselis::run;

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    run(event_loop, window).block_on()
}
