mod register;

use crate::cartridge::Cartridge;
use crate::nes::{SCREEN_HEIGHT, SCREEN_WIDTH};
use register::{AddressLatch, Register};
use std::sync::{Arc, RwLock};

const VBLANK_SCANLINE: u16 = 241;
const LAST_SCANLINE: u16 = 261;
const PIXEL_COUNT: usize = (SCREEN_HEIGHT * SCREEN_WIDTH * 3) as usize;

pub struct PPU {
    ppuctrl: u8,
    ppumask: u8,
    ppustatus: u8,
    oamaddr: u8,
    ppuscroll: u16,
    ppuaddr: u16,
    oamdma: u8,

    pub pattern_table_0: [u8; 0x1000],
    pub pattern_table_1: [u8; 0x1000],
    pub nametable_0: [u8; 0x0400],
    pub nametable_1: [u8; 0x0400],
    pub nametable_2: [u8; 0x0400],
    pub nametable_3: [u8; 0x0400],
    palette_ram_idx: [u8; 0x20],
    oam: [u8; 0x100],

    address_latch: AddressLatch,
    scroll_latch: AddressLatch,

    scanline: u16,

    cartridge: Arc<RwLock<Cartridge>>,

    pub screen: [u8; PIXEL_COUNT],
}

impl PPU {
    pub fn new(cartridge: Arc<RwLock<Cartridge>>) -> PPU {
        PPU {
            ppuctrl: 0x10,
            ppumask: 0,
            ppustatus: 0x10,
            oamaddr: 0x01,
            ppuscroll: 0,
            ppuaddr: 0x0001,
            oamdma: 0,

            pattern_table_0: [0; 0x1000],
            pattern_table_1: [0; 0x1000],
            nametable_0: [0; 0x0400],
            nametable_1: [0; 0x0400],
            nametable_2: [0; 0x0400],
            nametable_3: [0; 0x0400],
            palette_ram_idx: [0; 0x20],
            oam: [0; 0x100],

            address_latch: AddressLatch::HI,
            scroll_latch: AddressLatch::HI,

            scanline: 0,
            cartridge,

            screen: [0; PIXEL_COUNT],
        }
    }
}

impl PPU {
    pub fn tick(&mut self, cycles: u8) {
        for _ in 0..cycles {
            if self.scanline < (SCREEN_HEIGHT as u16) {
                self.render_scanline();
            }

            self.scanline += 1;

            if self.scanline == VBLANK_SCANLINE {
                self.set_vblank(true);
            } else if self.scanline == LAST_SCANLINE {
                self.scanline = 0;
                self.set_vblank(false);
            }
        }
    }

    // walks through the nametable to get the correct sprite index, then fetches that sprite from
    // the chr_rom and pushes the corresponding line of pixels into the screen.
    fn render_scanline(&mut self) {
        for x in 0..SCREEN_WIDTH {
            // each sprite is 8 pixels wide, so the chr index in the scanline is the position of
            // the pixel in the scanline divided by 8.
            let sprite_idx = (x / 8) + (self.scanline as usize / 8) * 32;
            let sprite_idx = self.nametable_0[sprite_idx];
            let sprite = self.cartridge.read().unwrap().chr_at(sprite_idx as usize);
            if sprite.is_empty() {
                continue;
            }

            // the position of the pixel we want from the sprite.
            let chr_x = x as u8 % 8;
            let chr_y = self.scanline as u8 % 8;
            let pixel = PPU::get_sprite_pixel(&sprite, chr_x, chr_y);

            // put pixel at screen's (x, scanline).
            let scanline = self.scanline as usize;
            self.set_pixel(x as usize, scanline, pixel);
        }
    }

    fn get_sprite_pixel(sprite: &[u8], x: u8, y: u8) -> u8 {
        let x = 7 - x;
        let base: u8 = 2;

        let line = sprite[y as usize];
        let lsb = line & base.pow(x as u32);

        let line = sprite[y as usize + 8];
        let msb = line & base.pow(x as u32);

        if lsb | msb > 0 {
            128
        } else {
            0
        }
    }

    pub fn get_vblank(&self) -> bool {
        self.ppustatus & 0x80 > 0
    }

    fn set_vblank(&mut self, val: bool) {
        if val {
            self.ppustatus = self.ppustatus | 0x80;
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, val: u8) {
        self.screen[(y * SCREEN_WIDTH + x) * 3 + 0] = val;
        self.screen[(y * SCREEN_WIDTH + x) * 3 + 1] = val;
        self.screen[(y * SCREEN_WIDTH + x) * 3 + 2] = val;
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
            0x0000..=0x1FFF => self.cartridge.read().unwrap().read(addr as u16),
            0x2000..=0x23FF => self.nametable_0[addr % 0x0400],
            0x2400..=0x27FF => self.nametable_1[addr % 0x0400],
            0x2800..=0x2BFF => self.nametable_2[addr % 0x0400],
            0x2C00..=0x2FFF => self.nametable_3[addr % 0x0400],
            0x3F00..=0x3F1F => self.palette_ram_idx[addr % 0x0020],
            _ => unimplemented!("PPU::readb at {:X}", addr),
        }
    }

    fn writeb(&mut self, addr: u16, val: u8) {
        let addr = PPU::map_addr(addr) as usize;
        match addr {
            0x0000..=0x1FFF => self.cartridge.write().unwrap().write(addr as u16, val),
            0x2000..=0x23FF => self.nametable_0[addr % 0x0400] = val,
            0x2400..=0x27FF => self.nametable_1[addr % 0x0400] = val,
            0x2800..=0x2BFF => self.nametable_2[addr % 0x0400] = val,
            0x2C00..=0x2FFF => self.nametable_3[addr % 0x0400] = val,
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
                self.address_latch = AddressLatch::HI;
                val
            }
            Register::OAMADDR => panic!("OAMADDR is write only"), // self.oamaddr,
            Register::OAMDATA => self.oam[self.oamaddr as usize],
            Register::PPUSCROLL => panic!("PPUSCROLL is write only"),
            Register::PPUADDR => panic!("PPUADDR is write only"),
            Register::PPUDATA => self.readb(self.ppuaddr),
            Register::OAMDMA => self.oamdma,
        }
    }

    pub fn set_nmi(&mut self) {
        self.ppustatus |= 0x80;
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        debug_assert!(addr <= 7);

        let reg: Register = (addr as usize).into();
        match reg {
            Register::PPUCTRL => self.ppuctrl = val,
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
                match self.scroll_latch {
                    AddressLatch::HI => self.ppuscroll = (self.ppuscroll & 0x00FF) | val << 8,
                    AddressLatch::LO => self.ppuscroll = (self.ppuscroll & 0xFF00) | val,
                };
                self.scroll_latch.next();
            }
            Register::PPUADDR => {
                let val = val as u16;
                match self.address_latch {
                    AddressLatch::HI => self.ppuaddr = (self.ppuaddr & 0x00FF) | val << 8,
                    AddressLatch::LO => self.ppuaddr = (self.ppuaddr & 0xFF00) | val,
                };
                self.address_latch.next();
            }
            Register::PPUDATA => {
                self.writeb(self.ppuaddr, val);
                self.incr_ppuaddr();
            }
            Register::OAMDMA => self.oamdma = val,
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

#[test]
fn test_get_sprite_pixel() {
    // 0 0 0 0 0 1 1 0
    // 0 0 0 0 0 1 1 0
    // 0 0 0 0 0 1 1 0
    // 0 0 0 0 0 1 1 0
    // 0 0 0 0 0 1 1 0
    // 0 0 0 0 0 1 1 0
    // 0 0 0 0 0 1 1 0
    // 0 0 0 0 0 1 1 0
    let data = vec![6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6];
    let data = &data;

    assert_eq!(
        &[0, 0, 0, 0, 0, 128, 128, 0],
        &[
            PPU::get_sprite_pixel(data, 0, 0),
            PPU::get_sprite_pixel(data, 1, 0),
            PPU::get_sprite_pixel(data, 2, 0),
            PPU::get_sprite_pixel(data, 3, 0),
            PPU::get_sprite_pixel(data, 4, 0),
            PPU::get_sprite_pixel(data, 5, 0),
            PPU::get_sprite_pixel(data, 6, 0),
            PPU::get_sprite_pixel(data, 7, 0),
        ],
    );
}
