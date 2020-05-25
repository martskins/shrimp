mod mapper;

use mapper::Mapper;
use std::io::Read;

pub struct Cartridge {
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn read(&self, addr: u16) -> u8 {
        self.mapper.readb(addr)
    }

    pub(crate) fn from_data(data: Vec<u8>) -> Cartridge {
        let mapper = mapper::from(data);
        let mapper = Box::new(mapper);
        Cartridge { mapper }
    }

    pub fn from_path(path: impl AsRef<str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = std::fs::File::open(path.as_ref())?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let mapper = mapper::from(data);
        let mapper = Box::new(mapper);
        Ok(Cartridge { mapper })
    }
}
