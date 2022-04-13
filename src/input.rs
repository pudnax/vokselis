use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    window::Window,
};

use crate::state::Uniform;

#[derive(Debug, Default)]
pub struct Input {
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub right_pressed: bool,
    pub left_pressed: bool,
    pub slash_pressed: bool,
    pub right_shift_pressed: bool,
    pub enter_pressed: bool,
    pub space_pressed: bool,
    pub left_mouse_pressed: bool,
    pub mouse_position: [f32; 2],
}

impl Input {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn update(&mut self, event: &WindowEvent, window: &Window) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(keycode),
                        state,
                        ..
                    },
                ..
            } => {
                let pressed = state == &ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Up => {
                        self.up_pressed = pressed;
                    }
                    VirtualKeyCode::Down => {
                        self.down_pressed = pressed;
                    }
                    VirtualKeyCode::Left => {
                        self.left_pressed = pressed;
                    }
                    VirtualKeyCode::Right => {
                        self.right_pressed = pressed;
                    }
                    VirtualKeyCode::Slash => {
                        self.slash_pressed = pressed;
                    }
                    VirtualKeyCode::RShift => {
                        self.right_shift_pressed = pressed;
                    }
                    VirtualKeyCode::Return => {
                        self.enter_pressed = pressed;
                    }
                    VirtualKeyCode::Space => {
                        self.space_pressed = pressed;
                    }
                    _ => return false,
                };
            }
            WindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => {
                let PhysicalSize { width, height } = window.inner_size();
                let x = (*x as f32 / width as f32 - 0.5) * 2.;
                let y = -(*y as f32 / height as f32 - 0.5) * 2.;
                self.mouse_position = [x, y];
            }
            WindowEvent::MouseInput {
                button: winit::event::MouseButton::Left,
                state,
                ..
            } => self.left_mouse_pressed = matches!(state, ElementState::Pressed),

            _ => {}
        }
        true
    }

    pub fn process_position(&self, uniform: &mut Uniform) {
        let dx = 0.01;
        if self.left_pressed {
            uniform.pos[0] -= dx;
        }
        if self.right_pressed {
            uniform.pos[0] += dx;
        }
        if self.down_pressed {
            uniform.pos[1] -= dx;
        }
        if self.up_pressed {
            uniform.pos[1] += dx;
        }
        if self.slash_pressed {
            uniform.pos[2] -= dx;
        }
        if self.right_shift_pressed {
            uniform.pos[2] += dx;
        }
        uniform.mouse_pressed = self.left_mouse_pressed as _;
        uniform.mouse = self.mouse_position;
    }
}
