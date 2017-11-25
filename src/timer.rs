pub struct Timer {
    m_clock: u8,
    base_clock: u64,
    divider: u8,

    pub timer_counter: u8,
    pub timer_modulo: u8,
    pub timer_enabled: bool,
    pub timer_speed: TimerSpeed,

    pub timer_interrupt: bool,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            m_clock: 0,
            base_clock: 0,
            divider: 0,
            timer_counter: 0,
            timer_modulo: 0,
            timer_enabled: false,
            timer_speed: TimerSpeed::ClockOver1024,
            timer_interrupt: false,
        }
    }

    pub fn increase(&mut self, cycles: u8) {
        assert!(cycles % 4 == 0);
        self.m_clock += cycles / 4;

        if self.m_clock >= 4 {
            self.m_clock -= 4;
            self.base_clock = self.base_clock.wrapping_add(1);

            if self.base_clock % 16 == 0 {
                self.divider = self.divider.wrapping_add(1);
            }

            if self.timer_enabled && self.base_clock % self.timer_speed.num_base_ticks() == 0 {
                if self.timer_counter == 0xFF {
                    self.timer_interrupt = true;
                    self.timer_counter = self.timer_modulo;
                } else {
                    self.timer_counter += 1;
                }
            }
        }
    }

    pub fn reset_divider(&mut self) {
        self.divider = 0;
        self.base_clock = 0;
    }

    pub fn get_divider(&self) -> u8 {
        self.divider
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

    fn num_base_ticks(&self) -> u64 {
        match *self {
            TimerSpeed::ClockOver1024 => 64,
            TimerSpeed::ClockOver16 => 1,
            TimerSpeed::ClockOver64 => 4,
            TimerSpeed::ClockOver256 => 16,
        }
    }
}
