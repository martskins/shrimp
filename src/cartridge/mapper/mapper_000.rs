use super::Header;

// NROM mapper implementation
pub struct Mapper {
    header: Header,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
}

impl Mapper {
    pub fn new(header: Header, data: Vec<u8>) -> Self {
        let prg_rom_size = header.prg_rom_size as usize * 0x4000;
        let prg_rom = data[..prg_rom_size].to_vec();

        let chr_rom_size = header.chr_rom_size as usize * 0x2000;
        let chr_rom = data[prg_rom_size..prg_rom_size + chr_rom_size].to_vec();

        Mapper {
            header,
            prg_rom,
            chr_rom,
        }
    }
}

impl super::Mapper for Mapper {
    fn readb(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => 0,
            0x8000..=0xBFFF => self.prg_rom[addr as usize - 0x8000],
            0xC000..=0xFFFF => {
                let addr = if self.header.prg_rom_size > 1 {
                    addr & 0x7FFF
                } else {
                    addr & 0x3FFF
                };
                self.prg_rom[addr as usize]
            }
            _ => 0,
        }
    }

    fn writeb(&mut self, addr: u16, val: u8) {}

    fn chr_at(&self, pos: usize) -> Vec<u8> {
        if self.chr_rom.is_empty() {
            return vec![];
        }

        self.chr_rom[pos * 16..(pos + 1) * 16].to_vec()
    }
}
