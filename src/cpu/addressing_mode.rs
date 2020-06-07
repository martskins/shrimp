use crate::cpu::CPU;

#[derive(Debug, Clone)]
pub(super) enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
}

impl AddressingMode {
    /// debump rolls back the program counter bump performed in the load operation of an
    /// AddressingMode. This should be used in any instruction that uses both am.load and am.store
    /// in the same block.
    pub(super) fn debump(&self, cpu: &mut CPU) {
        match self {
            AddressingMode::Implied => {}
            AddressingMode::Accumulator => {}
            AddressingMode::Immediate => cpu.reg.pc = cpu.reg.pc.wrapping_sub(1),
            AddressingMode::Absolute => cpu.reg.pc = cpu.reg.pc.wrapping_sub(2),
            AddressingMode::AbsoluteX => cpu.reg.pc = cpu.reg.pc.wrapping_sub(2),
            AddressingMode::AbsoluteY => cpu.reg.pc = cpu.reg.pc.wrapping_sub(2),
            AddressingMode::Indirect => cpu.reg.pc = cpu.reg.pc.wrapping_sub(2),
            AddressingMode::IndirectX => cpu.reg.pc = cpu.reg.pc.wrapping_sub(1),
            AddressingMode::IndirectY => cpu.reg.pc = cpu.reg.pc.wrapping_sub(1),
            AddressingMode::ZeroPage => cpu.reg.pc = cpu.reg.pc.wrapping_sub(1),
            AddressingMode::ZeroPageX => cpu.reg.pc = cpu.reg.pc.wrapping_sub(1),
            AddressingMode::ZeroPageY => cpu.reg.pc = cpu.reg.pc.wrapping_sub(1),
            AddressingMode::Relative => cpu.reg.pc = cpu.reg.pc.wrapping_sub(1),
        }
    }

    pub(super) fn load(&self, cpu: &mut CPU) -> u8 {
        match self {
            AddressingMode::Implied => panic!("invalid use of AddressingMode::Implied"),
            AddressingMode::Accumulator => cpu.reg.a,
            AddressingMode::Immediate => cpu.loadb_bump(),
            AddressingMode::Relative => {
                let offset = cpu.loadb_bump() as i8;
                let addr = (cpu.reg.pc as i16).wrapping_add(offset as i16);
                cpu.readb(addr as u16)
            }
            AddressingMode::ZeroPage => {
                let addr = cpu.loadb_bump() as u16;
                cpu.readb(addr)
            }
            AddressingMode::ZeroPageX => {
                let addr = (cpu.loadb_bump().wrapping_add(cpu.reg.x)) as u16;
                cpu.readb(addr)
            }
            AddressingMode::ZeroPageY => {
                let addr = (cpu.loadb_bump().wrapping_add(cpu.reg.y)) as u16;
                cpu.readb(addr)
            }
            AddressingMode::Absolute => {
                let addr = cpu.loadw_bump();
                cpu.readb(addr)
            }
            AddressingMode::AbsoluteX => {
                let addr = cpu.loadw_bump().wrapping_add(cpu.reg.x as u16);
                cpu.readb(addr)
            }
            AddressingMode::AbsoluteY => {
                let addr = cpu.loadw_bump().wrapping_add(cpu.reg.y as u16);
                cpu.readb(addr)
            }
            AddressingMode::Indirect => {
                let addr = cpu.loadw_bump();
                let addr = cpu.readw(addr);
                cpu.readb(addr)
            }
            AddressingMode::IndirectX => {
                let val = cpu.loadb_bump();
                let x = cpu.reg.x;
                let addr = cpu.readw_zp(val.wrapping_add(x));
                cpu.readb(addr)
            }
            AddressingMode::IndirectY => {
                let val = cpu.loadb_bump();
                let y = cpu.reg.y;
                let addr = cpu.readw_zp(val).wrapping_add(y as u16);
                cpu.readb(addr)
            }
        }
    }

    pub(super) fn store(&self, cpu: &mut CPU, val: u8) {
        match self {
            AddressingMode::Implied => panic!("invalid use of AddressingMode::Implied"),
            AddressingMode::Accumulator => cpu.reg.a = val,
            AddressingMode::Immediate => panic!("cannot store in AddressingMode::Immediate mode"),
            AddressingMode::Relative => {
                let offset = cpu.loadb_bump() as i8;
                let addr = (cpu.reg.pc as i16).wrapping_add(offset as i16);
                cpu.writeb(addr as u16, val);
            }
            AddressingMode::ZeroPage => {
                let addr = cpu.loadb_bump();
                cpu.writeb(addr as u16, val);
            }
            AddressingMode::ZeroPageX => {
                let addr = (cpu.loadb_bump().wrapping_add(cpu.reg.x)) as u16;
                cpu.writeb(addr, val);
            }
            AddressingMode::ZeroPageY => {
                let addr = (cpu.loadb_bump().wrapping_add(cpu.reg.y)) as u16;
                cpu.writeb(addr, val);
            }
            AddressingMode::Absolute => {
                let addr = cpu.loadw_bump();
                cpu.writeb(addr, val);
            }
            AddressingMode::AbsoluteX => {
                let addr = cpu.loadw_bump().wrapping_add(cpu.reg.x as u16);
                cpu.writeb(addr, val);
            }
            AddressingMode::AbsoluteY => {
                let addr = cpu.loadw_bump().wrapping_add(cpu.reg.y as u16);
                cpu.writeb(addr, val);
            }
            AddressingMode::Indirect => {
                let addr = cpu.loadw_bump();
                let addr = cpu.readw(addr);
                cpu.writeb(addr, val);
            }
            AddressingMode::IndirectX => {
                let x = cpu.reg.x;
                let addr = cpu.loadb_bump();
                let addr = cpu.readw_zp(addr.wrapping_add(x));
                cpu.writeb(addr, val);
            }
            AddressingMode::IndirectY => {
                let y = cpu.reg.y;
                let addr = cpu.loadb_bump();
                let addr = cpu.readw_zp(addr).wrapping_add(y as u16);
                cpu.writeb(addr, val);
            }
        };
    }
}
