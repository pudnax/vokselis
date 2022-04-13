use winit::event::{ElementState, VirtualKeyCode};

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
}

impl Input {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn update(&mut self, key: &VirtualKeyCode, state: &ElementState) -> bool {
        let pressed = state == &ElementState::Pressed;
        match key {
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
    }
}
