type MouseCoord = f64;

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
}

#[derive(Clone, Copy, Debug)]
pub struct InputData {
    pub mouse_pos: (MouseCoord, MouseCoord),
    pub mouse_pos_delta: (MouseCoord, MouseCoord),
}

impl InputData {
    pub fn new_from_prev(prev: &Self) -> Self {
        Self {
            mouse_pos: prev.mouse_pos,
            ..Default::default()
        }
    }
}

impl Default for InputData {
    fn default() -> Self {
        Self {
            mouse_pos: (0.0, 0.0),
            mouse_pos_delta: (0.0, 0.0),
        }
    }
}
