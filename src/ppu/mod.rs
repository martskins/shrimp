mod register;

use crate::cartridge::Cartridge;
use crate::{
    cpu::CPU,
    nes::{SCREEN_HEIGHT, SCREEN_WIDTH},
};
use register::{AddressLatch, Register};
use std::cell::RefCell;
use std::rc::Rc;

const VBLANK_SCANLINE: u16 = 241;
const LAST_SCANLINE: u16 = 261;
const PIXEL_COUNT: usize = (SCREEN_HEIGHT * SCREEN_WIDTH * 3) as usize;
const CYCLES_PER_SCANLINE: u64 = 114; // 29781 cycles per frame / 261 scanlines
static PALETTE: [u8; 192] = [
    124, 124, 124, 0, 0, 252, 0, 0, 188, 68, 40, 188, 148, 0, 132, 168, 0, 32, 168, 16, 0, 136, 20,
    0, 80, 48, 0, 0, 120, 0, 0, 104, 0, 0, 88, 0, 0, 64, 88, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 188,
    188, 0, 120, 248, 0, 88, 248, 104, 68, 252, 216, 0, 204, 228, 0, 88, 248, 56, 0, 228, 92, 16,
    172, 124, 0, 0, 184, 0, 0, 168, 0, 0, 168, 68, 0, 136, 136, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248,
    248, 248, 60, 188, 252, 104, 136, 252, 152, 120, 248, 248, 120, 248, 248, 88, 152, 248, 120,
    88, 252, 160, 68, 248, 184, 0, 184, 248, 24, 88, 216, 84, 88, 248, 152, 0, 232, 216, 120, 120,
    120, 0, 0, 0, 0, 0, 0, 252, 252, 252, 164, 228, 252, 184, 184, 248, 216, 184, 248, 248, 184,
    248, 248, 164, 192, 240, 208, 176, 252, 224, 168, 248, 216, 120, 216, 248, 120, 184, 248, 184,
    184, 248, 216, 0, 252, 252, 248, 216, 248, 0, 0, 0, 0, 0, 0,
];

const SPRITE_PALETTE_OFFSET: usize = 16;
const PALETTE_BASE: usize = 0x3F00;

#[derive(Default)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

enum SpritePriority {
    Front,
    Back,
}

#[derive(Debug, PartialEq)]
enum Flip {
    Horizontal,
    Vertical,
    Both,
    None,
}

struct SpritePixel {
    color: RGB,
    priority: SpritePriority,
    sprite_zero: bool,
}

struct Sprite {
    x: u8,
    y: u8,
    attributes: u8,
    tile_index: u8,
}

impl Sprite {
    fn palette(&self) -> u8 {
        (self.attributes & 0x03) << 2
    }

    fn priority(&self) -> SpritePriority {
        if self.attributes & 0x20 == 0 {
            SpritePriority::Front
        } else {
            SpritePriority::Back
        }
    }

    fn flip(&self) -> Flip {
        match (self.attributes & 0xC0) >> 6 {
            0x01 => Flip::Horizontal,
            0x10 => Flip::Vertical,
            0x11 => Flip::Both,
            _ => Flip::None,
        }
    }
}

pub struct PPU {
    ppuctrl: u8,
    ppumask: u8,
    ppustatus: u8,
    oamaddr: u8,
    ppuscroll: u16,
    ppuaddr: u16,
    cycles: u64,
    has_blanked: bool,
    // nametables is an array with 4 individual nametables, each one of them contains a value that
    // represents an index into the pattern table, which holds the sprite for each tile in the
    // brackground.
    nametables: [u8; 0x0400 * 4],
    // palette_ram_idx holds two spaces of 16 bytes, one for the background tiles and one for the
    // foreground (in that order), each byte represents an index into the PALETTE array.
    palette_ram_idx: [u8; 0x20],
    // TODO: this probably needs to live in the cartridge.
    // oam contains the addresses for the foreground sprites.
    oam: [u8; 0x100],

    address_latch: AddressLatch,
    // TODO: I think address and scroll share the same latch.
    // scroll_latch: AddressLatch,
    scanline: u16,

    cartridge: Rc<RefCell<Cartridge>>,

    // screen holds all the pixels in a frame, each frame is composed of 32x30 tiles, each of 8x8
    // pixels, for a total of (32 * 8  * 30 * 8) = (256 * 240) = PIXEL_COUNT.
    pub screen: [u8; PIXEL_COUNT],
    pub frame_complete: bool,
    ppudata_buffer: u8,
}

impl PPU {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>) -> Self {
        PPU {
            ppuctrl: 0x10,
            ppumask: 0,
            ppustatus: 0x10,
            oamaddr: 0x01,
            ppuscroll: 0,
            ppuaddr: 0x0001,
            address_latch: AddressLatch::HI,
            scanline: 0,
            frame_complete: false,

            nametables: [0; 0x0400 * 4],
            palette_ram_idx: [0; 0x20],
            oam: [0; 0x100],
            screen: [0; PIXEL_COUNT],
            cartridge,

            has_blanked: false,
            cycles: 0,
            ppudata_buffer: 0,
        }
    }

    pub fn tick(&mut self, cpu: &mut CPU) {
        self.frame_complete = false;

        loop {
            if self.cycles + CYCLES_PER_SCANLINE > cpu.cycles {
                break;
            }

            if self.scanline < (SCREEN_HEIGHT as u16) {
                self.render_scanline();
            }

            self.scanline += 1;

            if self.scanline == VBLANK_SCANLINE {
                self.set_vblank(true);
                self.ppustatus &= 0xBF;
                if self.vblank_nmi() {
                    cpu.nmi();
                }
            } else if self.scanline == LAST_SCANLINE {
                self.frame_complete = true;
                self.scanline = 0;
                self.set_vblank(false);
            }

            self.cycles += CYCLES_PER_SCANLINE;
        }
    }

    pub fn vblank_nmi(&self) -> bool {
        self.ppuctrl & 0x80 != 0
    }

    fn set_sprite_zero_hit(&mut self) {
        self.ppustatus |= 0x40;
    }

    fn render_background(&self) -> bool {
        self.ppumask & 0x08 > 0
    }

    fn render_background_leftmost(&self) -> bool {
        self.ppumask & 0x02 > 0
    }

    fn render_sprites(&self) -> bool {
        self.ppumask & 0x10 > 0
    }

    fn render_sprites_leftmost(&self) -> bool {
        self.ppumask & 0x04 > 0
    }

    fn foreground_offset(&self) -> u16 {
        if self.ppuctrl & 0x08 == 0 {
            0
        } else {
            0x1000
        }
    }

    fn background_offset(&self) -> u16 {
        if self.ppuctrl & 0x10 == 0 {
            0
        } else {
            0x1000
        }
    }

    fn set_sprite_overflow(&mut self, val: bool) {
        if val {
            self.ppustatus |= 0x40;
        } else {
            self.ppustatus &= !0x40;
        }
    }

    fn base_nametable(&self) -> u16 {
        match self.ppuctrl & 0x03 {
            0x00 => 0x2000,
            0x01 => 0x2400,
            0x02 => 0x2800,
            0x03 => 0x2C00,
            _ => unreachable!(),
        }
    }

    pub fn set_oam(&mut self, data: &[u8; 256]) {
        self.oam = *data;
    }

    // walks through the nametable to get the correct sprite index, then fetches that sprite from
    // the chr_rom and pushes the corresponding line of pixels into the screen.
    fn render_scanline(&mut self) {
        // pre-fetch both sprite and background tile data for this scanline.
        let visible_sprites = self.get_scanline_sprite_pixels();
        let scanline_tiles = self.get_scanline_background_pixels();

        for x in 0..SCREEN_WIDTH {
            let bg_pixel = self.get_background_pixel(&scanline_tiles, x as u8);
            let fg_pixel = self.get_sprite_pixel(&visible_sprites, x as u8);
            if let Some(ref fg_pixel) = fg_pixel {
                if fg_pixel.sprite_zero {
                    self.set_sprite_zero_hit();
                }
            }

            let pixel = match (bg_pixel, fg_pixel) {
                (None, None) => continue,
                (None, Some(fg)) => fg.color,
                (Some(bg), None) => bg,
                (
                    Some(bg),
                    Some(SpritePixel {
                        priority: SpritePriority::Back,
                        ..
                    }),
                ) => bg,
                (
                    Some(_),
                    Some(SpritePixel {
                        color,
                        priority: SpritePriority::Front,
                        ..
                    }),
                ) => color,
            };

            let scanline = self.scanline as usize;
            self.set_pixel(x as usize, scanline, pixel);
        }
    }

    // returns an array of 64 bytes, each representing a row of a background tile that is visible
    // on the current scanline.
    fn get_scanline_background_pixels(&mut self) -> [u8; 64] {
        let mut out = [0; 64];

        for i in 0..32 {
            // each sprite is 8 pixels wide, so the chr index in the scanline is the position of
            // the pixel in the scanline divided by 8.
            let chr_idx = i as u16 % 32 + ((self.scanline as u16 / 8) % 32) * 32;
            // read the chr_address from the nametable
            let base = self.base_nametable();
            let chr_address = 16 * self.readb(base + chr_idx) as u16;
            let chr_address = chr_address + self.scanline % 8;
            let chr_address = chr_address + self.background_offset();

            // load the two planes of the current tile's line
            let cartridge = self.cartridge.borrow();
            out[2 * i] = cartridge.read(chr_address);
            out[(2 * i) + 1] = cartridge.read(chr_address + 8);
        }

        out
    }

    fn get_scanline_sprite_pixels(&mut self) -> Vec<Sprite> {
        let mut out = vec![];
        for i in 0..64 {
            let i = i * 4;
            let sprite_y = self.oam[i].wrapping_add(1);
            let y = self.scanline;
            if y < sprite_y as u16 + 8 && y >= sprite_y as u16 {
                let sprite = Sprite {
                    // sprite data is delayed by one scanline, so we must add 1 to the y position
                    // of each sprite. See https://wiki.nesdev.com/w/index.php/PPU_OAM for more
                    // information on PPU OAM.
                    y: sprite_y,
                    tile_index: self.oam[i + 1],
                    attributes: self.oam[i + 2],
                    x: self.oam[i + 3],
                };

                if out.len() > 8 {
                    self.set_sprite_overflow(true);
                } else {
                    out.push(sprite);
                }
            }
        }

        out
    }

    fn get_sprite_pixel(&self, visible_sprites: &[Sprite], x: u8) -> Option<SpritePixel> {
        if !self.render_sprites() || (!self.render_sprites_leftmost() && x < 8) {
            return None;
        }

        let y = self.scanline;
        let cartridge = self.cartridge.borrow();
        for sprite in visible_sprites {
            if x >= sprite.x && x < sprite.x.wrapping_add(8) {
                let flip = sprite.flip();

                let chr_address = sprite.tile_index as u16 + self.foreground_offset();
                let y = y - sprite.y as u16;
                let mut chr_address = 16 * chr_address + y;
                if flip == Flip::Both || flip == Flip::Vertical {
                    chr_address = 7 - chr_address;
                }
                // load the two planes of the current tile's line
                let chr_left = cartridge.read(chr_address);
                let chr_right = cartridge.read(chr_address + 8);

                let x = x - sprite.x;
                let bit = if flip == Flip::Both || flip == Flip::Horizontal {
                    x % 8
                } else {
                    7 - (x % 8)
                };
                let (lsb, msb) = ((chr_left >> bit) & 0x01, (chr_right >> bit) & 0x01);
                let color_idx = (lsb | msb << 1) as u16;
                if color_idx == 0 {
                    continue;
                }

                let palette_index = sprite.palette();
                let palette_addr = PALETTE_BASE
                    + SPRITE_PALETTE_OFFSET
                    + palette_index as usize
                    + color_idx as usize;
                let color_addr = self.readb(palette_addr as u16) as usize & 0x3F;
                return Some(SpritePixel {
                    color: RGB {
                        r: PALETTE[color_addr * 3],
                        g: PALETTE[color_addr * 3 + 1],
                        b: PALETTE[color_addr * 3 + 2],
                    },
                    priority: sprite.priority(),
                    sprite_zero: chr_address < 0x03,
                });
            } else {
                continue;
            }
        }

        None
    }

    // takes a &[u8; 64], representing the pixels for the current scanline, and returns the pixel
    // color that should be display at position (x, scanline).
    fn get_background_pixel(&self, tiles: &[u8; 64], x: u8) -> Option<RGB> {
        if !self.render_background() || (!self.render_background_leftmost() && x < 8) {
            return None;
        }

        let index = (x as usize / 8) * 2;
        let chr_left = tiles[index];
        let chr_right = tiles[index + 1];

        let bit = 7 - (x % 8);
        let (lsb, msb) = ((chr_left >> bit) & 0x01, (chr_right >> bit) & 0x01);
        let color_idx = (lsb | msb << 1) as u16;

        let attr_byte = self.get_attr_byte(x, self.scanline);
        let (left, top) = (x % 32 < 16, self.scanline % 32 < 16);
        let palette_offset = match (left, top) {
            (true, true) => attr_byte & 0x03,
            (false, true) => (attr_byte >> 2) & 0x03,
            (true, false) => (attr_byte >> 4) & 0x03,
            (false, false) => (attr_byte >> 6) & 0x03,
        };
        let palette_index = palette_offset << 2;
        debug_assert!(palette_index as u16 | color_idx < 0x20);

        let palette_addr = PALETTE_BASE + palette_index as usize + color_idx as usize;
        let color_addr = self.readb(palette_addr as u16) as usize & 0x3F;
        Some(RGB {
            r: PALETTE[color_addr * 3],
            g: PALETTE[color_addr * 3 + 1],
            b: PALETTE[color_addr * 3 + 2],
        })
    }

    fn get_attr_byte(&self, x: u8, y: u16) -> u8 {
        let x = x as u16 / 32;
        let y = y / 32;
        let base = self.base_nametable();
        self.readb(base + 0x3C0 + x + (y * 8))
    }

    // pub fn get_vblank(&mut self) -> bool {
    //     self.ppustatus & 0x80 > 0
    // }

    pub fn set_vblank(&mut self, val: bool) {
        self.has_blanked = true;

        if val {
            self.ppustatus |= 0x80;
        } else {
            self.ppustatus &= !0x80;
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, val: RGB) {
        self.screen[(y * SCREEN_WIDTH + x) * 3] = val.b;
        self.screen[(y * SCREEN_WIDTH + x) * 3 + 1] = val.g;
        self.screen[(y * SCREEN_WIDTH + x) * 3 + 2] = val.r;
    }

    fn map_addr(addr: u16) -> u16 {
        let addr = addr % 0x4000;
        match addr {
            0x3000..=0x3EFF => addr - 0x1000,
            0x3F20..=0x3FFF => ((addr - 0x3F00) % 0x0020) + 0x3F00,
            _ => addr,
        }
    }

    fn readb(&self, addr: u16) -> u8 {
        let addr = PPU::map_addr(addr) as usize;
        match addr {
            // addresses 0x0000 to 0x1FFF are mapped to the pattern table, which can reside in the
            // PPU RAM or the cartridge's ROM.
            0x0000..=0x1FFF => self.cartridge.borrow().read(addr as u16),
            0x2000..=0x2FFF => self.nametables[addr % 0x0400],
            0x3F00..=0x3F1F => self.palette_ram_idx[addr % 0x0020],
            _ => unimplemented!("PPU::readb at {:X}", addr),
        }
    }

    fn writeb(&mut self, addr: u16, val: u8) {
        let addr = PPU::map_addr(addr) as usize;
        match addr {
            0x0000..=0x1FFF => self.cartridge.borrow_mut().write(addr as u16, val),
            0x2000..=0x2FFF => self.nametables[addr % 0x0400] = val,
            0x3F00..=0x3F1F => self.palette_ram_idx[addr % 0x0020] = val,
            _ => unimplemented!("PPU::writeb at {:X}", addr),
        }
    }

    fn incr_ppuaddr(&mut self) {
        let inc = if (self.ppuctrl & 0x04) == 0 { 1 } else { 32 };
        self.ppuaddr = self.ppuaddr.wrapping_add(inc as u16);
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        debug_assert!(addr <= 7);

        let reg: Register = (addr as usize).into();
        match reg {
            Register::PPUCTRL => self.ppuctrl,
            Register::PPUMASK => self.ppumask,
            Register::PPUSTATUS => {
                let val = self.ppustatus;
                self.ppustatus &= 0x7F;
                self.address_latch = AddressLatch::HI;
                // self.scroll_latch = AddressLatch::HI;
                val
            }
            Register::OAMADDR => panic!("OAMADDR is write only"), // self.oamaddr,
            Register::OAMDATA => self.oam[self.oamaddr as usize],
            Register::PPUSCROLL => panic!("PPUSCROLL is write only"),
            Register::PPUADDR => panic!("PPUADDR is write only"),
            Register::PPUDATA => {
                let addr = self.ppuaddr;
                let val = self.readb(addr);
                self.incr_ppuaddr();
                if addr < 0x3F00 {
                    let buffered_val = self.ppudata_buffer;
                    self.ppudata_buffer = val;
                    buffered_val
                } else {
                    val
                }
            }
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        debug_assert!(addr <= 7);

        let reg: Register = (addr as usize).into();
        match reg {
            Register::PPUCTRL => {
                // self.address_latch = AddressLatch::HI;
                // self.ppustatus &= 0x7F;
                self.ppuctrl = val
            }
            Register::PPUMASK => self.ppumask = val,
            Register::PPUSTATUS => {
                // self.address_latch.next();
            }
            Register::OAMADDR => self.oamaddr = val,
            Register::OAMDATA => {
                self.oam[self.oamaddr as usize] = val;
                self.oamaddr = self.oamaddr.wrapping_add(1);
            }
            Register::PPUSCROLL => {
                let val = val as u16;
                match self.address_latch {
                    AddressLatch::HI => self.ppuscroll = (self.ppuscroll & 0x00FF) | val << 8,
                    AddressLatch::LO => self.ppuscroll = (self.ppuscroll & 0xFF00) | val,
                };
                self.address_latch.next();
            }
            Register::PPUADDR => {
                let val = val as u16;
                match self.address_latch {
                    AddressLatch::HI => self.ppuaddr = (self.ppuaddr & 0x00FF) | val << 8,
                    AddressLatch::LO => self.ppuaddr = (self.ppuaddr & 0xFF00) | val,
                };

                // TODO: cpu_dummy_writes/cpu_dummy_writes_ppumem.nes fails with:
                //      A single write to $2006 must not change the address used by $2007 when
                //      vblank is on.
                //
                // I assume we need to set something like this, but it still fails with it.
                //      if !self.get_vblank() {
                //          self.address_latch.next();
                //      }

                self.address_latch.next();
            }
            Register::PPUDATA => {
                self.writeb(self.ppuaddr, val);
                self.incr_ppuaddr();
            }
        }

        match reg {
            Register::PPUADDR
            | Register::PPUSCROLL
            | Register::PPUCTRL
            | Register::PPUDATA
            | Register::PPUMASK
            | Register::PPUSTATUS => {
                self.ppustatus &= 0b1110_0000;
                self.ppustatus |= 0b0001_1111 & val;
            }
            _ => {}
        }
    }
}
