use super::RESET_VECTOR;

pub enum Register {
    A,
    X,
    Y,
    PC,
    S,
    P,
}

#[derive(Debug)]
pub struct Registers {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub s: u8,
    pub p: u8,
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0xFD,
            p: 0,
        }
    }
}

impl Registers {
    pub fn set_reg(&mut self, reg: Register, val: u8) {
        match reg {
            Register::A => self.a = val,
            Register::X => self.x = val,
            Register::Y => self.y = val,
            Register::S => self.s = val,
            // P => self.a = val,
            _ => panic!("invalid use of set_reg"),
        }
    }

    pub fn get_reg(&self, reg: Register) -> u8 {
        match reg {
            Register::A => self.a,
            Register::X => self.x,
            Register::Y => self.y,
            Register::S => self.s,
            // P => self.a = val,
            _ => panic!("invalid use of set_reg"),
        }
    }

    pub fn set_flag(&mut self, flag: Flag, val: bool) {
        let sum = match flag {
            Flag::N => 0b1000_0000,
            Flag::V => 0b0100_0000,
            Flag::B => 0b0001_0000,
            Flag::D => 0b0000_1000,
            Flag::I => 0b0000_0100,
            Flag::Z => 0b0000_0010,
            Flag::C => 0b0000_0001,
        };

        if val {
            self.p |= sum;
        } else {
            self.p &= !sum;
        }
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::N => (self.p & 0b1000_0000) > 0,
            Flag::V => (self.p & 0b0100_0000) > 0,
            Flag::B => (self.p & 0b0001_0000) > 0,
            Flag::D => (self.p & 0b0000_1000) > 0,
            Flag::I => (self.p & 0b0000_0100) > 0,
            Flag::Z => (self.p & 0b0000_0010) > 0,
            Flag::C => (self.p & 0b0000_0001) > 0,
        }
    }
}

pub enum Flag {
    N,
    V,
    B,
    D,
    I,
    Z,
    C,
}

#[derive(Debug, Default)]
pub struct Flags {
    n: bool,
    v: bool,
    d: bool,
    i: bool,
    z: bool,
    c: bool,
}
