use std::path::{Path, PathBuf};
use std::time::Instant;

mod state;
mod utils;
mod watcher;

use color_eyre::eyre::Result;
use pollster::FutureExt;
use utils::input::Input;
use utils::{frame_counter::FrameCounter, recorder::RecordEvent};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

const SHADER_FOLDER: &str = "shaders";

pub async fn run(
    event_loop: EventLoop<(PathBuf, wgpu::ShaderModule)>,
    window: Window,
) -> Result<()> {
    let mut state = state::State::new(&window, &event_loop).block_on()?;

    let mut recording_status = false;
    let recorder = utils::recorder::Recorder::new();

    print_help(state.get_info(), &recorder.ffmpeg_version);

    let mut frame_counter = FrameCounter::new();
    let mut input = Input::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::MainEventsCleared => {
                state.update(&frame_counter, &input);
                window.request_redraw();
            }
            Event::WindowEvent { event, .. } => {
                input.update(&event, &window);
                match event {
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

                    WindowEvent::Resized(PhysicalSize { width, height })
                    | WindowEvent::ScaleFactorChanged {
                        new_inner_size: &mut PhysicalSize { width, height },
                        ..
                    } => {
                        if width != 0 && height != 0 {
                            state.resize(width, height);
                        }

                        if recording_status {
                            println!("Stop recording. Resolution has been changed.",);
                            recording_status = false;
                            recorder.send(RecordEvent::Finish);
                        }
                    }

                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    } => {
                        if VirtualKeyCode::F11 == keycode {
                            let now = Instant::now();
                            let frame = state.capture_frame();
                            eprintln!("Capture image: {:#.2?}", now.elapsed());
                            recorder.send(RecordEvent::Screenshot(frame));
                        }
                        if VirtualKeyCode::F12 == keycode {
                            if recording_status {
                                recorder.send(RecordEvent::Finish);
                            } else {
                                recorder.send(RecordEvent::Start(dbg!(
                                    state.screenshot_ctx.image_dimentions
                                )));
                            }
                            recording_status = !recording_status;
                        }
                    }

                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                frame_counter.record();
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

                if recording_status {
                    let (frame, _) = state.capture_frame();
                    recorder.send(RecordEvent::Record(frame));
                }
            }
            Event::UserEvent((path, shader)) => state.register_shader_change(path, shader),
            Event::LoopDestroyed => {
                println!("\n// End from the loop. Bye bye~⏎ ");
            }
            _ => {}
        }
    })
}

pub fn print_help(info: impl std::fmt::Display, ffmpeg_version: &str) {
    println!("{}", info);
    println!("{}", ffmpeg_version);
    println!(
        "Default shader path:\n\t{}\n",
        Path::new(SHADER_FOLDER).canonicalize().unwrap().display()
    );
    // println!("\n- `F1`:   Print help");
    // println!("- `F2`:   Toggle play/pause");
    // println!("- `F3`:   Pause and step back one frame");
    // println!("- `F4`:   Pause and step forward one frame");
    // println!("- `F5`:   Restart playback at frame 0 (`Time` and `Pos` = 0)");
    // println!("- `F6`:   Print parameters");
    // println!("- `F7`:   Toggle profiler");
    // println!("- `F8`:   Switch backend");
    // println!("- `F10`:  Save shaders");
    println!("- `F11`:  Take Screenshot");
    println!("- `F12`:  Start/Stop record video");
    println!("- `ESC`:  Exit the application");
    // println!("- `Arrows`: Change `Pos`");
    println!();
    println!("// Set up our new world⏎ ");
    println!("// And let's begin the⏎ ");
    println!("\tSIMULATION⏎ \n");
}
