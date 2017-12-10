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
    large_sprite: bool,
    priority: bool,
    x_flip: bool,
    y_flip: bool,
    palette: Palette,
}

impl SpriteAttribute {
    fn from_memory(memory: [u8; 4], large_sprite: bool) -> SpriteAttribute {
        let x_position = memory[1];
        let y_position = memory[0];
        let tile_num = if large_sprite {
            memory[2] & 0b1111_1110
        } else {
            memory[2]
        };
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
            large_sprite,
            priority,
            x_flip,
            y_flip,
            palette,
        }
    }

    pub fn get_x_pos(&self) -> u8 {
        self.x_position
    }

    pub fn get_y_pos(&self) -> u8 {
        self.y_position
    }

    pub fn get_tile_num(&self) -> u8 {
        self.tile_num
    }

    pub fn is_large_sprite(&self) -> bool {
        self.large_sprite
    }

    pub fn has_priority(&self) -> bool {
        self.priority
    }

    pub fn has_x_flip(&self) -> bool {
        self.x_flip
    }

    pub fn has_y_flip(&self) -> bool {
        self.y_flip
    }

    pub fn get_palette(&self) -> Palette {
        self.palette
    }
}

pub fn get_sprite_attributes_from_oam(
    oam: &BlockMemory,
    large_sprites: bool,
) -> Vec<SpriteAttribute> {
    let mut attributes = Vec::new();
    attributes.reserve_exact(NUM_SPRITES as usize);
    for i in 0..NUM_SPRITES {
        let from = i * 0x4;
        attributes.push(SpriteAttribute::from_memory(
            oam.read_4_bytes(from),
            large_sprites,
        ));
    }
    attributes
}
