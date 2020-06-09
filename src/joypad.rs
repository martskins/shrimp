const A: u8 = 0;
const B: u8 = 1;
const SELECT: u8 = 2;
const START: u8 = 3;
const UP: u8 = 4;
const DOWN: u8 = 5;
const LEFT: u8 = 6;
const RIGHT: u8 = 7;

// See https://wiki.nesdev.com/w/index.php/Standard_controller for more information on how the NES
// joypad behaves.
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
        if self.strobe < 8 {
            self.strobe += 1;
        }
    }

    pub fn reset(&mut self) {
        self.strobe = 0;
    }

    pub fn state(&mut self) -> bool {
        // Each read reports one bit at a time through D0. The first 8 reads will indicate which
        // buttons or directions are pressed (1 if pressed, 0 if not pressed). All subsequent reads
        // will return 1 on official Nintendo brand controllers but may return 0 on third party
        // controllers such as the U-Force.
        if self.strobe == 8 {
            return true;
        }

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
