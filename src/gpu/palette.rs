#[derive(Copy, Clone)]
pub enum Palette {
    BackgroundPalette,
    ObjectPalette0,
    ObjectPalette1,
}

pub struct Palettes {
    pub bg: u8,
    pub obj0: u8,
    pub obj1: u8,
}

impl Palettes {
    pub fn new() -> Palettes {
        Palettes { bg: 0, obj0: 0, obj1: 0 }
    }
}

pub fn apply_palette(color: u8, palette: Palette, palettes: &Palettes) -> u8 {
    let palette = match palette {
        Palette::BackgroundPalette => palettes.bg,
        Palette::ObjectPalette0 => palettes.obj0,
        Palette::ObjectPalette1 => palettes.obj1,
    };
    palette >> (color * 2) & 3
}

