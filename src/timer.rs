pub struct Timer {
    internal_counter: u16,

    pub timer_counter: u8,
    pub timer_modulo: u8,
    pub timer_enabled: bool,
    pub timer_speed: TimerSpeed,

    pub timer_interrupt: bool,

    tima_reload_timer: Option<u8>,
    old_tima_relevant_bit: bool,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            internal_counter: 0,
            timer_counter: 0,
            timer_modulo: 0,
            timer_enabled: false,
            timer_speed: TimerSpeed::ClockOver1024,
            timer_interrupt: false,
            tima_reload_timer: None,

            old_tima_relevant_bit: false,
        }
    }

    pub fn increase(&mut self, cycles: u8) {
        for _ in 0..cycles as u16 {
            let new_counter = self.internal_counter.wrapping_add(1);
            self.change_counter_to(new_counter);
            if let Some(left) = self.tima_reload_timer {
                if left == 0 {
                    self.tima_reload_timer = None;
                    self.timer_counter = self.timer_modulo;
                    self.timer_interrupt = true;
                } else {
                    self.tima_reload_timer = Some(left - 1);
                }
            }
        }
    }

    pub fn reset_divider(&mut self) {
        self.change_counter_to(0);
    }

    pub fn get_divider(&self) -> u8 {
        ((self.internal_counter & 0xFF00) >> 8) as u8
    }

    fn change_counter_to(&mut self, to: u16) {
        self.internal_counter = to;
        let bit = 1 << self.timer_speed.relevant_counter_bit();
        let before = self.old_tima_relevant_bit;
        let after = (self.internal_counter & bit != 0) && self.timer_enabled;
        let falling_edge = before && !after;
        if falling_edge {
            if self.timer_counter == 0xFF {
                self.tima_reload_timer = Some(3);
            }
            self.timer_counter = self.timer_counter.wrapping_add(1);
        }
        self.old_tima_relevant_bit = after;
    }
}

pub enum TimerSpeed {
    ClockOver1024,
    ClockOver16,
    ClockOver64,
    ClockOver256,
}

impl TimerSpeed {
    pub fn to_u8(&self) -> u8 {
        match *self {
            TimerSpeed::ClockOver1024 => 0,
            TimerSpeed::ClockOver16 => 1,
            TimerSpeed::ClockOver64 => 2,
            TimerSpeed::ClockOver256 => 3,
        }
    }

    pub fn from_u8(from: u8) -> TimerSpeed {
        match from {
            0 => TimerSpeed::ClockOver1024,
            1 => TimerSpeed::ClockOver16,
            2 => TimerSpeed::ClockOver64,
            3 => TimerSpeed::ClockOver256,
            _ => panic!("Not a valid TimerSpeed u8"),
        }
    }

    fn relevant_counter_bit(&self) -> usize {
        match *self {
            TimerSpeed::ClockOver1024 => 9,
            TimerSpeed::ClockOver16 => 3,
            TimerSpeed::ClockOver64 => 5,
            TimerSpeed::ClockOver256 => 7,
        }
    }
}
