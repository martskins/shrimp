mod cartridge;
mod cpu;
mod nes;
mod ppu;

use nes::NES;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path: String = std::env::args().skip(1).take(1).collect();
    let mut nes = NES::new(path);
    nes.run()
}
