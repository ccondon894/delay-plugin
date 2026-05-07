pub struct DelayLine {
    buffer: Vec<f32>,
    write_index: usize,
}

impl DelayLine {
    pub fn new(max_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; max_samples + 1], 
            write_index: 0,
        }
        
    }
    // return sample at read index
    pub fn read(&self, delay_samples: usize) -> f32 {
        let read_index = (self.write_index + self.buffer.len() - delay_samples) % self.buffer.len();
        self.buffer[read_index]
    }
    
    // store sample at current write position
    // advance write index so the next write does to the next slot
    pub fn write(&mut self, sample: f32) {
        self.buffer[self.write_index] = sample; 
        self.write_index = (self.write_index + 1) % self.buffer.len();
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.write_index = 0;
    }
}