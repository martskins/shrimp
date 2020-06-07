mod cartridge;
mod cpu;
mod joypad;
mod nes;
mod ppu;

use nes::NES;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(short = "r", long)]
    rom: String,
    #[structopt(short = "s", long, default_value = "1")]
    scale: u8,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Options::from_args();
    let mut nes = NES::new(opts);
    nes.run()
}
