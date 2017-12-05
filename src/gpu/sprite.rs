use gpu::*;

#[derive(Copy, Clone)]
pub struct SpritePixel {
    pub pixel: Option<u8>,
    pub palette: Option<Palette>,
    pub priority: bool,
}

pub struct SpriteAttribute {
    x_position: u8,
    y_position: u8,
    tile_num: u8,
    priority: bool,
    x_flip: bool,
    y_flip: bool,
    palette: Palette,
}

impl SpriteAttribute {
    fn from_memory(memory: [u8; 4]) -> SpriteAttribute {
        let x_position = memory[1];
        let y_position = memory[0];
        let tile_num = memory[2];
        let flags = memory[3];
        let priority = flags & 0x80 == 0;
        let x_flip = flags & 0x20 != 0;
        let y_flip = flags & 0x40 != 0;
        let palette = if flags & 0x10 != 0 {
            Palette::ObjectPalette1
        } else {
            Palette::ObjectPalette0
        };
        SpriteAttribute {
            x_position,
            y_position,
            tile_num,
            priority,
            x_flip,
            y_flip,
            palette,
        }
    }

    pub fn get_tile(&self) -> Tile {
        Tile {
            tile_num: self.tile_num,
            tile_set: TileSet::Set1,
            x_flip: self.x_flip,
            y_flip: self.y_flip,
            _large_tile: false,
        }
    }

    pub fn get_x_pos(&self) -> u8 {
        self.x_position
    }

    pub fn get_y_pos(&self) -> u8 {
        self.y_position
    }

    pub fn has_priority(&self) -> bool {
        self.priority
    }

    pub fn get_palette(&self) -> Palette {
        self.palette
    }
}

pub fn get_sprite_attributes_from_oam(oam: &BlockMemory) -> Vec<SpriteAttribute> {
    let mut attributes = Vec::new();
    attributes.reserve_exact(NUM_SPRITES as usize);
    for i in 0..NUM_SPRITES {
        let from = i * 0x4;
        attributes.push(SpriteAttribute::from_memory(oam.read_4_bytes(from)));
    }
    attributes
}

