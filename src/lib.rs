use std::{path::PathBuf, time::Instant};

mod frame_counter;
mod shader_compiler;
mod state;
mod utils;
mod watcher;

use color_eyre::eyre::Result;
use pollster::FutureExt;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub async fn run(
    event_loop: EventLoop<(PathBuf, wgpu::ShaderModule)>,
    window: Window,
) -> Result<()> {
    let mut state = state::State::new(&window, &event_loop).block_on()?;

    let mut last_frame_inst = Instant::now();
    let mut frame_counter = frame_counter::FrameCounter::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::MainEventsCleared => {
                window.request_redraw();
            }
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
                    if width != 0 && height != 0 {
                        state.resize(width, height);
                    }
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
            Event::UserEvent((path, module)) => {
                if let Some(pipeline) = state.watcher.hash_dump.get_mut(&path) {
                    let mut pipeline = pipeline.borrow_mut();
                    pipeline.reload(&state.device, module);
                }
            }
            _ => {}
        }
    })
}
