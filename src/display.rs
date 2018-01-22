use sdl2::Sdl;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

pub const COLS: usize = 160;
const ROWS: usize = 144;
pub const PIXELS: usize = (COLS * ROWS) as usize;
const SCALE_FACTOR: u32 = 5;

pub trait Display {
    fn redraw(&mut self);
    fn clear(&mut self);
    fn set_line(&mut self, line: u8, pixels: &[u8; COLS]);
}

pub struct SdlDisplay {
    pixels: [u8; PIXELS],
    display_context: SdlDisplayContext,
    texture: Texture,
}

impl SdlDisplay {
    pub fn new(sdl_context: &Sdl) -> SdlDisplay {
        let display_context = SdlDisplayContext::new(sdl_context);
        let texture = display_context
            .texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, COLS as u32, ROWS as u32)
            .unwrap();
        SdlDisplay {
            pixels: [0; PIXELS],
            display_context,
            texture,
        }
    }
}

impl Display for SdlDisplay {
    fn redraw(&mut self) {
        let pixels = &self.pixels;
        self.texture
            .with_lock(None, |buffer: &mut [u8], _: usize| for (i, &p) in
                pixels.iter().enumerate()
            {
                let offset = i * 3;
                let val = (3 - p as u8) * (255 / 3);
                buffer[offset] = val;
                buffer[offset + 1] = val;
                buffer[offset + 2] = val;
            })
            .unwrap();
        self.display_context.canvas.clear();
        self.display_context
            .canvas
            .copy(&self.texture, None, None)
            .unwrap();
        self.display_context.canvas.present();
    }

    fn clear(&mut self) {
        self.pixels = [0; PIXELS];
    }

    fn set_line(&mut self, line: u8, pixels: &[u8; COLS]) {
        let line = line as usize;
        self.pixels[(COLS * line)..(COLS * (line + 1))].copy_from_slice(pixels);
    }
}

struct SdlDisplayContext {
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
}

impl SdlDisplayContext {
    pub fn new(sdl_context: &Sdl) -> SdlDisplayContext {
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(
                "RustGB",
                SCALE_FACTOR * COLS as u32,
                SCALE_FACTOR * ROWS as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().unwrap();
        let texture_creator = canvas.texture_creator();
        SdlDisplayContext {
            canvas,
            texture_creator,
        }
    }
}
