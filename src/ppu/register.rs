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

#[derive(Debug, Eq, PartialEq)]
pub enum Register {
    PPUCTRL,   // 0x2000
    PPUMASK,   // 0x2001
    PPUSTATUS, // 0x2002
    OAMADDR,   // 0x2003
    OAMDATA,   // 0x2004
    PPUSCROLL, // 0x2005
    PPUADDR,   // 0x2006
    PPUDATA,   // 0x2007
    #[allow(unused)]
    OAMDMA, // 0x2008
}

impl From<usize> for Register {
    fn from(n: usize) -> Register {
        let n = if n >= 0x2000 { n - 0x2000 } else { n };

        match n & 7 {
            0 => Register::PPUCTRL,
            1 => Register::PPUMASK,
            2 => Register::PPUSTATUS,
            3 => Register::OAMADDR,
            4 => Register::OAMDATA,
            5 => Register::PPUSCROLL,
            6 => Register::PPUADDR,
            7 => Register::PPUDATA,
            // TODO: chech whether this is needed
            // 8 => Reg::OAMDMA,
            _ => panic!("not a valid PPU reg"),
        }
    }
}
