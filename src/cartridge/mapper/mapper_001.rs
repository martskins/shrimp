use super::Header;

pub struct Mapper {
    shift_register: u8,
    must_write_register: bool,
    header: Header,
    prg_rom_size: usize,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    chr_bank_1: usize,
    chr_bank_2: usize,
    prg_bank: usize,

    prg_offsets: [u32; 2],
    chr_offsets: [u32; 2],
    control: u8,
}

impl Mapper {
    pub fn new(header: Header, data: Vec<u8>) -> Mapper {
        let prg_rom_size = 0x4000 * header.prg_rom_size;
        let (prg_rom, chr_rom) = data.split_at(prg_rom_size);
        Mapper {
            shift_register: 0x10,
            must_write_register: false,
            header,
            prg_rom_size,
            prg_rom: prg_rom.to_vec(),
            chr_rom: chr_rom.to_vec(),
            chr_bank_1: 0,
            chr_bank_2: 0,
            prg_bank: 0,
            prg_offsets: [0; 2],
            chr_offsets: [0; 2],
            control: 0,
        }
    }

    fn write_shift_register(&mut self, addr: u16, val: u8) {
        if val >= 0x80 {
            self.shift_register = 0x10;
        } else {
            let done = self.shift_register & 0x01 == 0x01;
            let bit = (val & 0x01) << 4;
            self.shift_register >>= 1;
            self.shift_register |= bit;

            // when a 1 is pushed into the first bit the register should be written in the
            // next write attempt.
            if done {
                match addr {
                    // 0x8000..=0x9FFF => m.writeControl(value),
                    // 0x9FFF..=0xBFFF => m.writeCHRBank0(value),
                    // 0xBFFF..=0xDFFF => m.writeCHRBank1(value),
                    0x0000..=0x7FFF => unreachable!(),
                    0x8000..=0xDFFE => {}
                    0xDFFF..=0xFFFF => {
                        self.prg_bank = (val & 0x0F) as usize;
                    }
                }

                self.shift_register = 0x10;
                self.update_offsets();
            }
        }
    }

    fn update_offsets(&mut self) {
        match (self.control & 0x0C) >> 2 {
            0 | 1 => {
                self.prg_offsets[0] = self.prg_offset((self.prg_bank as u32) & 0x0E);
                self.prg_offsets[1] = self.prg_offset(((self.prg_bank as u32) | 0x01) & 0x0F);
            }
            2 => {
                self.prg_offsets[0] = 0;
                self.prg_offsets[1] = self.prg_offset((self.prg_bank as u32) & 0x0F);
            }
            3 => {
                self.prg_offsets[0] = self.prg_offset((self.prg_bank as u32) & 0x0F);
                self.prg_offsets[1] = self.prg_offset((self.prg_rom.len() as u32) / 0x4000 - 1);
            }
            _ => panic!("Invalid prg control value: {:b}", self.control),
        }

        match (self.control & 0x10) >> 4 {
            0 => {
                self.chr_offsets[0] = self.chr_offset((self.chr_bank_1 as u32) & 0x1E);
                self.chr_offsets[1] = self.chr_offset((self.chr_bank_1 as u32) | 0x01);
            }
            1 => {
                self.chr_offsets[0] = self.chr_offset((self.chr_bank_1 as u32) & 0x1F);
                self.chr_offsets[1] = self.chr_offset((self.chr_bank_2 as u32) & 0x1F);
            }
            _ => panic!("Invalid chr control value: {:b}", self.control),
        }
    }

    fn prg_offset(&self, index: u32) -> u32 {
        (index % ((self.prg_rom.len() as u32) / 0x4000)) * 0x4000
    }

    fn chr_offset(&self, index: u32) -> u32 {
        0
        // (index % ((self.chr_rom.len() as u32) / 0x1000)) * 0x1000
    }
}

impl super::Mapper for Mapper {
    fn writeb(&mut self, addr: u16, val: u8) {
        match addr {
            0x4020..=0x5FFF => {
                print!("{}", val as char);
            }
            0x6000..=0x6003 => {}
            0x8000..=0xFFFF => self.write_shift_register(addr, val),
            x => {} // x => panic!("write at {:X}", x),
        }
    }

    fn readb(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let bank_offset = self.chr_bank_1 * 0x2000;
                self.chr_rom[bank_offset + addr as usize]
            }
            0x4020..=0x5FFF => 0,
            0x6000..=0x7FFF => 0,
            0x8000..=0xFFFF => {
                let addr = addr - 0x8000;
                let bank = addr / 0x4000;
                let offset = addr % 0x4000;
                let addr = self.prg_offsets[bank as usize] + (offset as u32);
                self.prg_rom[addr as usize]
            }
            _ => unimplemented!("cMMC1 read"),
        }
    }

    fn chr_at(&self, pos: usize) -> &[u8] {
        if self.chr_rom.is_empty() {
            return &[];
        }

        &self.chr_rom[pos * 16..(pos + 1) * 16]
    }
}

#[test]
fn test_write_shift_register() {
    use crate::cartridge::mapper::Mapper;

    let header = Header {
        prg_rom_size: 1,
        chr_rom_size: 0,
        mapper: 1,
    };
    let data = [0; 0x16000].to_vec();
    let mut m = super::mapper_001::Mapper::new(header, data);

    m.writeb(0xE000, 0x01); // 0b0001_1000;
    assert_eq!(m.shift_register, 0b0001_1000);

    m.writeb(0xE000, 0x00); // 0b0000_1100;
    assert_eq!(m.shift_register, 0b0000_1100);

    m.writeb(0xE000, 0x01); // 0b0001_0110;
    assert_eq!(m.shift_register, 0b0001_0110);

    m.writeb(0xE000, 0x00); // 0b0000_1011;
    assert_eq!(m.shift_register, 0b0000_1011);

    m.writeb(0xE000, 0x01); // shift register is reset to 0x10
    assert_eq!(m.shift_register, 0b0001_0000);
}
