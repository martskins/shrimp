use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::ppu::PPU;
use std::sync::{Arc, RwLock};

pub struct NES {
    cpu: CPU,
    cartridge: Arc<RwLock<Cartridge>>,
}

impl NES {
    pub fn new(path: impl AsRef<str>) -> Self {
        let cartridge = Cartridge::from_path(path.as_ref()).unwrap();
        let cartridge = Arc::new(RwLock::new(cartridge));
        let cpu = CPU::new(cartridge.clone());
        Self { cpu, cartridge }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.cpu.do_loop();
        Ok(())
    }
}
