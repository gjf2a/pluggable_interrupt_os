use pc_keyboard::DecodedKey;

pub struct HandlerTable {
    timer: Option<fn()>, keyboard: Option<fn(DecodedKey)>
}

impl HandlerTable {
    pub fn new() -> Self {
        HandlerTable {timer: None, keyboard: None}
    }

    pub fn timer(mut self, timer_handler: fn()) -> Self {
        self.timer = Some(timer_handler);
        self
    }

    pub fn handle_timer(&self) {
        if let Some(timer) = self.timer {
            (timer)()
        }
    }

    pub fn keyboard(mut self, keyboard_handler: fn(DecodedKey)) -> Self {
        self.keyboard = Some(keyboard_handler);
        self
    }

    pub fn handle_keyboard(&self, key: DecodedKey) {
        if let Some(keyboard) = self.keyboard {
            (keyboard)(key)
        }
    }
}