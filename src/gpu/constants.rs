pub const SCANLINE_OAM_TIME: u32 = 80;
pub const SCANLINE_VRAM_TIME: u32 = 172;
pub const HORIZONTAL_BLANK_TIME: u32 = 204;
pub const VERTICAL_BLANK_TIME: u32 = 4560;
pub const DIM_X: usize = 160;
pub const DIM_Y: u8 = 144;
pub const OFFSET_TILE_SET_1: u16 = 0x0000;
// Offset to the 0th tile. There are negative tile numbers in tile set 1
pub const OFFSET_TILE_SET_0: u16 = 0x1000;
pub const OFFSET_TILE_MAP_0: u16 = 0x1800;
pub const OFFSET_TILE_MAP_1: u16 = 0x1C00;
pub const TILE_SIZE_IN_BYTES: u16 = 0x10;
pub const CLOCK_TICKS_PER_FRAME: u32 = 70224;
pub const NUM_SPRITES: u16 = 40;
