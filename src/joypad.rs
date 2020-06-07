const A: u8 = 0;
const B: u8 = 1;
const SELECT: u8 = 2;
const START: u8 = 3;
const UP: u8 = 4;
const DOWN: u8 = 5;
const LEFT: u8 = 6;
const RIGHT: u8 = 7;

#[derive(Debug, Default)]
pub struct Joypad {
    pub a: bool,
    pub b: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub start: bool,
    pub select: bool,

    strobe: u8,
}

impl Joypad {
    fn next(&mut self) {
        self.strobe = (self.strobe + 1) % 8;
    }

    pub fn reset(&mut self) {
        self.strobe = 0;
    }

    pub fn state(&mut self) -> bool {
        let val = match self.strobe {
            A => self.a,
            B => self.b,
            START => self.start,
            SELECT => self.select,
            UP => self.up,
            DOWN => self.down,
            LEFT => self.left,
            RIGHT => self.right,
            _ => false,
        };

        self.next();
        val
    }
}
