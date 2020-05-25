use super::Header;

pub struct Mapper {
    header: Header,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
}

impl Mapper {
    pub fn new(header: Header, data: Vec<u8>) -> Self {
        let (prg_rom, chr_rom) = data.split_at(header.prg_rom_size as usize * 0x4000);
        let prg_rom = prg_rom.to_vec();
        let chr_rom = chr_rom.to_vec();
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
                let res = self.prg_rom[addr as usize];
                res
            }
            _ => 0,
        }
    }
}
