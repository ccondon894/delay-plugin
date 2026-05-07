pub struct OnePoleLowPass {
    prev_output: f32,
}

impl OnePoleLowPass {
    pub fn new() -> Self {
        Self {
            prev_output: 0.0,
        }
    }

    pub fn process(&mut self, input: f32, a: f32) -> f32 {
        let filtered = (1.0 - a) * input + a * self.prev_output;
        self.prev_output = filtered;
        filtered
    }

    pub fn clear(&mut self) {
        self.prev_output = 0.0;
    }
}