pub struct VolumeEnvelope {
    volume: u8,
    envelope_starting_volume: u8,
    volume_envelope_direction: bool,
    volume_envelope_period: u8,
    volume_envelope_tick_counts: u8,
}

impl VolumeEnvelope {
    pub fn new() -> VolumeEnvelope {
        VolumeEnvelope {
            volume: 15,
            envelope_starting_volume: 0,
            volume_envelope_direction: false,
            volume_envelope_period: 0,
            volume_envelope_tick_counts: 0,
        }
    }

    pub fn apply_volume_control(&self, sample: i16) -> i16 {
        sample * self.volume as i16 * (8000/15)
    }

    pub fn set_starting_volume(&mut self, value: u8) {
        assert!(value < 0x10);
        self.envelope_starting_volume = value;
        self.volume = value;
    }

    pub fn get_starting_volume(&self) -> u8 {
        self.envelope_starting_volume
    }

    pub fn set_direction(&mut self, direction: bool) {
        self.volume_envelope_direction = direction;
    }

    pub fn get_direction(&self) -> bool {
        self.volume_envelope_direction
    }

    pub fn set_period(&mut self, period: u8) {
        self.volume_envelope_period = period;
        self.volume_envelope_tick_counts = period;
    }

    pub fn get_period(&self) -> u8 {
        self.volume_envelope_period
    }

    pub fn tick(&mut self) {
        if self.volume_envelope_period == 0 { return; }
        if self.volume_envelope_tick_counts > 0 {
            self.volume_envelope_tick_counts -= 1;
        }
        if self.volume_envelope_tick_counts == 0 {
            self.volume_envelope_tick_counts = self.volume_envelope_period;
            if self.volume > 0 && !self.volume_envelope_direction {
                self.volume -= 1;
            }
            if self.volume < 15 && self.volume_envelope_direction {
                self.volume += 1;
            }
        }
    }
}
