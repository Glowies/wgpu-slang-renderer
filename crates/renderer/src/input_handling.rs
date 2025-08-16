use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta},
};

type MouseCoord = f64;

/// Enum representing the state of a button, with an inner u32
/// that indicates how many frames that state has been active for.
#[derive(Clone, Copy, Debug)]
pub enum ButtonState {
    Released(u32),
    Pressed(u32),
}

impl ButtonState {
    fn increment(&self) -> Self {
        match self {
            ButtonState::Released(val) => ButtonState::Released(*val + 1),
            ButtonState::Pressed(val) => ButtonState::Pressed(*val + 1),
        }
    }

    fn update_from_element_state(&mut self, state: ElementState, prev: Self) {
        match (prev, state) {
            (Self::Released(_), ElementState::Pressed) => {
                *self = Self::Pressed(0);
            }
            (Self::Pressed(_), ElementState::Released) => {
                *self = Self::Released(0);
            }
            (..) => {}
        }
    }
}

pub struct Input {
    curr: InputData,
    prev: InputData,
}

impl Input {
    pub fn new() -> Self {
        Self {
            curr: InputData::default(),
            prev: InputData::default(),
        }
    }

    pub fn reset_frame(&mut self) {
        self.prev = self.curr;
        self.curr = InputData::new_from_prev(&self.prev);
    }

    pub fn data(&self) -> &InputData {
        &self.curr
    }

    pub fn handle_cursor_moved(&mut self, new_pos: (MouseCoord, MouseCoord)) {
        self.curr.mouse_pos = new_pos;
        self.curr.mouse_pos_delta = (
            new_pos.0 - self.prev.mouse_pos.0,
            new_pos.1 - self.prev.mouse_pos.1,
        );
    }

    pub fn handle_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        match (button, state) {
            (MouseButton::Left, button_state) => {
                self.curr
                    .mouse_button_left
                    .update_from_element_state(button_state, self.prev.mouse_button_left);
            }
            (MouseButton::Right, button_state) => {
                self.curr
                    .mouse_button_right
                    .update_from_element_state(button_state, self.prev.mouse_button_right);
            }
            _ => {}
        }
    }

    pub fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => self.curr.mouse_wheel_delta += y,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => {
                self.curr.mouse_wheel_delta += y as f32
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InputData {
    pub mouse_pos: (MouseCoord, MouseCoord),
    pub mouse_pos_delta: (MouseCoord, MouseCoord),
    pub mouse_wheel_delta: f32,
    pub mouse_button_left: ButtonState,
    pub mouse_button_right: ButtonState,
}

impl InputData {
    pub fn new_from_prev(prev: &Self) -> Self {
        Self {
            mouse_pos: prev.mouse_pos,
            mouse_button_left: prev.mouse_button_left.increment(),
            mouse_button_right: prev.mouse_button_right.increment(),
            ..Default::default()
        }
    }
}

impl Default for InputData {
    fn default() -> Self {
        Self {
            mouse_pos: (0.0, 0.0),
            mouse_pos_delta: (0.0, 0.0),
            mouse_wheel_delta: 0.0,
            mouse_button_left: ButtonState::Released(0),
            mouse_button_right: ButtonState::Released(0),
        }
    }
}
