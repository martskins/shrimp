#![allow(clippy::cast_lossless)]

mod register;
use register::{AddressLatch, Reg};

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

    cycles: usize,
    address_latch: AddressLatch,
}

impl PPU {
    pub fn new() -> PPU {
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

            cycles: 0,
            address_latch: AddressLatch::HI,
        }
    }
}

impl PPU {
    pub fn tick(&mut self, cycles: usize) {
        self.cycles = 3 * cycles;
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
            0x0000..=0x0FFF => self.pattern_table_0[addr % 0x1000],
            0x1000..=0x1FFF => self.pattern_table_1[addr % 0x1000],
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
            0x0000..=0x0FFF => self.pattern_table_0[addr % 0x1000] = val,
            0x1000..=0x1FFF => self.pattern_table_1[addr % 0x1000] = val,
            0x2000..=0x23FF => self.nametable_0[addr % 0x0400] = val,
            0x2400..=0x27FF => self.nametable_1[addr % 0x0400] = val,
            0x2800..=0x2BFF => self.nametable_2[addr % 0x0400] = val,
            0x2C00..=0x2FFF => self.nametable_3[addr % 0x0400] = val,
            0x3F00..=0x3F1F => self.palette_ram_idx[addr % 0x0020] = val,
            _ => unimplemented!("PPU::writeb at {:X}", addr),
        }
    }

    fn incr_ppuaddr(&mut self) {
        let inc = (self.ppuctrl & 0x04) >> 2;
        self.ppuaddr = self.ppuaddr.wrapping_add(inc as u16);
    }

    pub fn read(&mut self, reg: impl Into<Reg>) -> u8 {
        let reg = reg.into();
        match reg {
            Reg::PPUCTRL => self.ppuctrl,
            Reg::PPUMASK => self.ppumask,
            Reg::PPUSTATUS => {
                let val = self.ppustatus;
                // self.ppustatus &= 0xEF;
                self.address_latch = AddressLatch::HI;
                val
            }
            Reg::OAMADDR => self.oamaddr,
            Reg::OAMDATA => self.oam[self.oamaddr as usize],
            Reg::PPUSCROLL => panic!("PPUSCROLL is write only"),
            Reg::PPUADDR => panic!("PPUADDR is write only"),
            Reg::PPUDATA => self.readb(self.ppuaddr),
            Reg::OAMDMA => self.oamdma,
        }
    }

    pub fn set_nmi(&mut self) {
        self.ppustatus |= 0x80;
    }

    pub fn write(&mut self, reg: impl Into<Reg>, val: u8) {
        let reg = reg.into();
        match reg {
            Reg::PPUCTRL => {
                // if self.cycles <= 29658 * 3 {
                //     return;
                // }

                self.ppuctrl = val;
            }
            Reg::PPUMASK => {
                // if self.cycles <= 29658 * 3 {
                //     return;
                // }
                self.ppumask = val;
            }
            Reg::PPUSTATUS => {
                self.address_latch.next();
            }
            Reg::OAMADDR => self.oamaddr = val,
            Reg::OAMDATA => {
                self.oam[self.oamaddr as usize] = val;
                self.oamaddr = self.oamaddr.wrapping_add(1);
            }
            Reg::PPUSCROLL => {
                // if self.cycles <= 29658 * 3 {
                //     return;
                // }

                let val = val as u16;
                match self.address_latch {
                    AddressLatch::HI => self.ppuscroll = (self.ppuscroll & 0x00FF) | val << 8,
                    AddressLatch::LO => self.ppuscroll = (self.ppuscroll & 0xFF00) | val,
                };
                self.address_latch.next();
            }
            Reg::PPUADDR => {
                // if self.cycles <= 29658 * 3 {
                //     return;
                // }

                let val = val as u16;
                match self.address_latch {
                    AddressLatch::HI => self.ppuaddr = (self.ppuaddr & 0x00FF) | val << 8,
                    AddressLatch::LO => self.ppuaddr = (self.ppuaddr & 0xFF00) | val,
                };
                self.address_latch.next();
            }
            Reg::PPUDATA => {
                self.writeb(self.ppuaddr, val);
                self.incr_ppuaddr();
            }
            Reg::OAMDMA => self.oamdma = val,
        }

        match reg {
            Reg::PPUADDR
            | Reg::PPUSCROLL
            | Reg::PPUCTRL
            | Reg::PPUDATA
            | Reg::PPUMASK
            | Reg::PPUSTATUS => {
                self.ppustatus &= 0b1110_0000;
                self.ppustatus |= 0b0001_1111 & val;
            }
            _ => {}
        }
    }
}
