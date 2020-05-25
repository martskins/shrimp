mod nrom_000;

pub trait Mapper {
    fn readb(&self, addr: u16) -> u8;

    fn readw(&self, addr: u16) -> u16 {
        let lo = self.readb(addr) as u16;
        let hi = self.readb(addr) as u16;
        (hi << 8) | lo
    }
}

pub struct Header {
    /// prg rom size in 16kb units
    prg_rom_size: usize,
    /// chr rom size in 8kb units
    chr_rom_size: usize,
    mapper: u8,
}

fn read_header(data: [u8; 16]) -> Header {
    Header {
        prg_rom_size: data[4] as usize,
        chr_rom_size: data[5] as usize,
        mapper: (data[7] & 0x80) | (data[6] >> 4),
    }
}

pub fn from(data: Vec<u8>) -> impl Mapper {
    // let header_data = &data[0..=15];
    let (header_data, data) = data.split_at(16);
    let mut header: [u8; 16] = [0; 16];
    header.copy_from_slice(&header_data[0..=15]);
    let header = read_header(header);
    match header.mapper {
        0x00 => nrom_000::Mapper::new(header, data.to_vec()),
        n => panic!("unimeplemented mapper {}", n),
    }
}
