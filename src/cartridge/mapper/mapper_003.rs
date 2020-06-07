use super::Header;

pub struct Mapper {
    header: Header,
    prg_rom_size: usize,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    selected_bank: usize,
}

impl Mapper {
    pub fn new(header: Header, data: Vec<u8>) -> Mapper {
        let prg_rom_size = header.prg_rom_size as usize;
        let (prg_rom, chr_rom) = data.split_at(0x4000 * prg_rom_size);
        Mapper {
            header,
            prg_rom_size,
            prg_rom: prg_rom.to_vec(),
            chr_rom: chr_rom.to_vec(),
            selected_bank: 0,
        }
    }
}

impl super::Mapper for Mapper {
    fn writeb(&mut self, addr: u16, val: u8) {
        match addr {
            0x4020..=0x5FFF => {
                print!("{}", val as char);
            }
            0x6000..=0x7FFF => {
                print!("{}", val as char);
            }
            0x8000..=0xFFFF => self.selected_bank = (addr & 0x03) as usize,
            _ => panic!("not implemented"),
        }
    }

    fn readb(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let bank_offset = self.selected_bank * 0x2000;
                self.chr_rom[bank_offset + addr as usize]
            }
            0x4020..=0x5FFF => 0,
            0x6000..=0x7FFF => 0,
            0x8000..=0xFFFF => {
                let addr = addr as usize - 0x8000;
                self.prg_rom[addr % self.prg_rom_size]
            }
            _ => unimplemented!("cnrom read {:X}", addr),
        }
    }
}
