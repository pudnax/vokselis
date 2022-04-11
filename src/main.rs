use std::time::Instant;

use color_eyre::eyre::Result;
use pollster::FutureExt;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod state;

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    let mut state = state::State::new(&window).block_on()?;

    let mut last_frame_inst = Instant::now();
    let mut frame_counter = FrameCounter::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(PhysicalSize { width, height }) => {
                    state.resize(width, height);
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                frame_counter.record(&mut last_frame_inst);
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        state.resize(state.width, state.height);
                        window.request_redraw();
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => {
                        eprintln!("{:?}", e);
                        window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    })
}

struct FrameCounter {
    frame_count: u32,
    accum_time: f32,
}

impl FrameCounter {
    fn new() -> Self {
        Self {
            frame_count: 0,
            accum_time: 0.,
        }
    }

    fn record(&mut self, current_instant: &mut Instant) -> f32 /* dt */ {
        self.accum_time += current_instant.elapsed().as_secs_f32();
        *current_instant = Instant::now();
        self.frame_count += 1;
        if self.frame_count == 100 {
            println!(
                "Avg frame time {}ms",
                self.accum_time * 1000.0 / self.frame_count as f32
            );
            self.accum_time = 0.0;
            self.frame_count = 0;
        }
        self.accum_time
    }
}
