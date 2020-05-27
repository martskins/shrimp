use super::Header;

pub struct Mapper {
    shift_register: u8,
    header: Header,
    prg_rom_size: usize,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    selected_bank: usize,
}

impl Mapper {
    pub fn new(header: Header, data: Vec<u8>) -> Mapper {
        let prg_rom_size = 0x4000 * header.prg_rom_size;
        let (prg_rom, chr_rom) = data.split_at(prg_rom_size);
        Mapper {
            shift_register: 0,
            header,
            prg_rom_size,
            prg_rom: prg_rom.to_vec(),
            chr_rom: chr_rom.to_vec(),
            selected_bank: 0,
        }
    }

    fn write_reg(addr: u16, val: u8) {
        // match addr {
        // }
    }
}

impl super::Mapper for Mapper {
    fn writeb(&mut self, addr: u16, val: u8) {
        match addr {
            0x4020..=0x5FFF => {
                print!("{}", val as char);
            }
            0x6000..=0x6003 => {}
            0x8000..=0xFFFF => {
                if val < 0x80 {
                    let done = self.shift_register & 0x01 == 0x01;
                    self.shift_register >>= 1;
                    self.shift_register |= (val & 0x01) << 4;
                    if done {
                        self.shift_register = 0x10;
                    }
                } else {
                    self.shift_register = 0x10;
                }
            }
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
            _ => unimplemented!("cMMC1 read"),
        }
    }
}
