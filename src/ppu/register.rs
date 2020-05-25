#[derive(Debug)]
pub(super) enum AddressLatch {
    LO,
    HI,
}

impl AddressLatch {
    pub(super) fn next(&mut self) {
        match self {
            AddressLatch::LO => *self = AddressLatch::HI,
            AddressLatch::HI => *self = AddressLatch::LO,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Reg {
    PPUCTRL,
    PPUMASK,
    PPUSTATUS,
    OAMADDR,
    OAMDATA,
    PPUSCROLL,
    PPUADDR,
    PPUDATA,
    OAMDMA,
}

impl From<usize> for Reg {
    fn from(n: usize) -> Reg {
        let n = if n >= 0x2000 { n - 0x2000 } else { n };

        match n % 0x0009 {
            0 => Reg::PPUCTRL,
            1 => Reg::PPUMASK,
            2 => Reg::PPUSTATUS,
            3 => Reg::OAMADDR,
            4 => Reg::OAMDATA,
            5 => Reg::PPUSCROLL,
            6 => Reg::PPUADDR,
            7 => Reg::PPUDATA,
            8 => Reg::OAMDMA,
            _ => panic!("not a valid PPU reg"),
        }
    }
}
