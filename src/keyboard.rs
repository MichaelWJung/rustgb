use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const KEY_UP: Keycode = Keycode::Up;
const KEY_DOWN: Keycode = Keycode::Down;
const KEY_LEFT: Keycode = Keycode::Left;
const KEY_RIGHT: Keycode = Keycode::Right;
const KEY_A: Keycode = Keycode::U;
const KEY_B: Keycode = Keycode::I;
const KEY_START: Keycode = Keycode::Return;
const KEY_SELECT: Keycode = Keycode::Space;

fn key_to_index(keycode: Keycode) -> Option<usize> {
    match keycode {
        KEY_UP     => Some(0x0),
        KEY_DOWN   => Some(0x1),
        KEY_LEFT   => Some(0x2),
        KEY_RIGHT  => Some(0x3),
        KEY_A      => Some(0x4),
        KEY_B      => Some(0x5),
        KEY_START  => Some(0x6),
        KEY_SELECT => Some(0x7),
        _ => None
    }
}

pub struct Keyboard<'a> {
    key_statuses: [bool; 8],
    event_pump: &'a mut EventPump,
}

impl<'a> Keyboard<'a> {
    pub fn new(event_pump: &'a mut EventPump) -> Keyboard<'a> {
        Keyboard {
            key_statuses: [false; 8],
            event_pump,
        }
    }

    pub fn update_key_register(&self, register: &mut u8) {
        *register |= 0xF;
        if *register & 0x10 != 0 {
            if self.key_statuses[key_to_index(KEY_A).unwrap()]      { *register &= !0x1; }
            if self.key_statuses[key_to_index(KEY_B).unwrap()]      { *register &= !0x2; }
            if self.key_statuses[key_to_index(KEY_SELECT).unwrap()] { *register &= !0x4; }
            if self.key_statuses[key_to_index(KEY_START).unwrap()]  { *register &= !0x8; }
        }
        if *register & 0x20 != 0 {
            if self.key_statuses[key_to_index(KEY_RIGHT).unwrap()]  { *register &= !0x1; }
            if self.key_statuses[key_to_index(KEY_LEFT).unwrap()]   { *register &= !0x2; }
            if self.key_statuses[key_to_index(KEY_UP).unwrap()]     { *register &= !0x4; }
            if self.key_statuses[key_to_index(KEY_DOWN).unwrap()]   { *register &= !0x8; }
        }
    }

    pub fn check_events(&mut self) {
        while let Some(event) = self.event_pump.poll_event() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => ::std::process::exit(0),
                Event::KeyDown { keycode: Some(key), .. } => self.key_down(key),
                Event::KeyUp { keycode: Some(key), .. } => self.key_up(key),
                _ => {}
            }
        }
    }

    fn key_down(&mut self, keycode: Keycode) {
        let index = key_to_index(keycode);
        if let Some(key) = index {
            self.key_statuses[key] = true;
        }
    }

    fn key_up(&mut self, keycode: Keycode) {
        let index = key_to_index(keycode);
        if let Some(key) = index {
            self.key_statuses[key] = false;
        }
    }

}
