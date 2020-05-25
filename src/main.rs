mod cartridge;
mod cpu;
mod ppu;

use cartridge::Cartridge;
use cpu::CPU;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path: String = std::env::args().skip(1).take(1).collect();
    let cartridge = Cartridge::from_path(path)?;
    let mut cpu = CPU::new(&cartridge);
    cpu.do_loop();
    Ok(())
}
