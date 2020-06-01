mod addressing_mode;
mod register;

use crate::cartridge::Cartridge;
use crate::cpu::addressing_mode::AddressingMode;
use crate::ppu::PPU;
use register::{Flag, Registers};
use std::cell::RefCell;
#[cfg(feature = "debug")]
use std::io::Write;
use std::rc::Rc;

const NMI_VECTOR: u16 = 0xfffa;
const RESET_VECTOR: u16 = 0xfffc;
const BRK_VECTOR: u16 = 0xfffe;

pub struct CPU {
    reg: Registers,
    ram: [u8; 0x0800],
    apu: [u8; 0x0018],
    ppu: Rc<RefCell<PPU>>,
    cartridge: Rc<RefCell<Cartridge>>,
    #[cfg(feature = "debug")]
    logger: std::fs::File,
}

impl CPU {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>, ppu: Rc<RefCell<PPU>>) -> Self {
        #[cfg(feature = "debug")]
        let file = std::fs::File::create("log.txt").unwrap();
        let mut cpu = CPU {
            reg: Registers::default(),
            ram: [0; 0x0800],
            apu: [0; 0x0018],
            ppu,
            cartridge,
            #[cfg(feature = "debug")]
            logger: file,
        };
        cpu.reset();
        cpu
    }

    pub fn reset(&mut self) {
        self.reg.pc = self.readw(RESET_VECTOR);
        self.reg.p = 0x24;
    }

    pub fn tick(&mut self) -> u8 {
        #[cfg(feature = "debug")]
        let pc = self.reg.pc;

        let opcode = self.loadb_bump();

        #[cfg(feature = "debug")]
        writeln!(
            &mut self.logger,
            "{:04X} {:02X} \t\t A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            pc, opcode, self.reg.a, self.reg.x, self.reg.y, self.reg.p, self.reg.s,
        )
        .unwrap();
        match opcode {
            0x69 => self.adc(AddressingMode::Immediate),
            0x65 => self.adc(AddressingMode::ZeroPage),
            0x75 => self.adc(AddressingMode::ZeroPageX),
            0x6D => self.adc(AddressingMode::Absolute),
            0x7D => self.adc(AddressingMode::AbsoluteX),
            0x79 => self.adc(AddressingMode::AbsoluteY),
            0x61 => self.adc(AddressingMode::IndirectX),
            0x71 => self.adc(AddressingMode::IndirectY),

            0x29 => self.and(AddressingMode::Immediate),
            0x25 => self.and(AddressingMode::ZeroPage),
            0x35 => self.and(AddressingMode::ZeroPageX),
            0x2D => self.and(AddressingMode::Absolute),
            0x3D => self.and(AddressingMode::AbsoluteX),
            0x39 => self.and(AddressingMode::AbsoluteY),
            0x21 => self.and(AddressingMode::IndirectX),
            0x31 => self.and(AddressingMode::IndirectY),

            0x0A => self.asl(AddressingMode::Accumulator),
            0x06 => self.asl(AddressingMode::ZeroPage),
            0x16 => self.asl(AddressingMode::ZeroPageX),
            0x0E => self.asl(AddressingMode::Absolute),
            0x1E => self.asl(AddressingMode::AbsoluteX),

            0x24 => self.bit(AddressingMode::ZeroPage),
            0x2C => self.bit(AddressingMode::Absolute),

            0x90 => self.bcc(AddressingMode::Relative),
            0xB0 => self.bcs(AddressingMode::Relative),
            0xF0 => self.beq(AddressingMode::Relative),
            0x30 => self.bmi(AddressingMode::Relative),
            0xD0 => self.bne(AddressingMode::Relative),
            0x10 => self.bpl(AddressingMode::Relative),
            0x00 => self.brk(AddressingMode::Implied),
            0x50 => self.bvc(AddressingMode::Relative),
            0x70 => self.bvs(AddressingMode::Relative),

            0x18 => self.clc(AddressingMode::Implied),
            0xD8 => self.cld(AddressingMode::Implied),
            0x58 => self.cli(AddressingMode::Implied),
            0xB8 => self.clv(AddressingMode::Implied),

            0xC9 => self.cmp(AddressingMode::Immediate),
            0xC5 => self.cmp(AddressingMode::ZeroPage),
            0xD5 => self.cmp(AddressingMode::ZeroPageX),
            0xCD => self.cmp(AddressingMode::Absolute),
            0xDD => self.cmp(AddressingMode::AbsoluteX),
            0xD9 => self.cmp(AddressingMode::AbsoluteY),
            0xC1 => self.cmp(AddressingMode::IndirectX),
            0xD1 => self.cmp(AddressingMode::IndirectY),

            0xE0 => self.cpx(AddressingMode::Immediate),
            0xE4 => self.cpx(AddressingMode::ZeroPage),
            0xEC => self.cpx(AddressingMode::Absolute),
            0xC0 => self.cpy(AddressingMode::Immediate),
            0xC4 => self.cpy(AddressingMode::ZeroPage),
            0xCC => self.cpy(AddressingMode::Absolute),

            0xC6 => self.dec(AddressingMode::ZeroPage),
            0xD6 => self.dec(AddressingMode::ZeroPageX),
            0xCE => self.dec(AddressingMode::Absolute),
            0xDE => self.dec(AddressingMode::AbsoluteX),
            0xCA => self.dex(AddressingMode::Implied),
            0x88 => self.dey(AddressingMode::Implied),

            0x49 => self.eor(AddressingMode::Immediate),
            0x45 => self.eor(AddressingMode::ZeroPage),
            0x55 => self.eor(AddressingMode::ZeroPageX),
            0x4D => self.eor(AddressingMode::Absolute),
            0x5D => self.eor(AddressingMode::AbsoluteX),
            0x59 => self.eor(AddressingMode::AbsoluteY),
            0x41 => self.eor(AddressingMode::IndirectX),
            0x51 => self.eor(AddressingMode::IndirectY),

            0xE6 => self.inc(AddressingMode::ZeroPage),
            0xF6 => self.inc(AddressingMode::ZeroPageX),
            0xEE => self.inc(AddressingMode::Absolute),
            0xFE => self.inc(AddressingMode::AbsoluteX),
            0xE8 => self.inx(AddressingMode::Implied),
            0xC8 => self.iny(AddressingMode::Implied),

            0x4C => self.jmp(AddressingMode::Absolute),
            0x6C => self.jmp(AddressingMode::Indirect),
            0x20 => self.jsr(AddressingMode::Absolute),

            0xA9 => self.lda(AddressingMode::Immediate),
            0xA5 => self.lda(AddressingMode::ZeroPage),
            0xB5 => self.lda(AddressingMode::ZeroPageX),
            0xAD => self.lda(AddressingMode::Absolute),
            0xBD => self.lda(AddressingMode::AbsoluteX),
            0xB9 => self.lda(AddressingMode::AbsoluteY),
            0xA1 => self.lda(AddressingMode::IndirectX),
            0xB1 => self.lda(AddressingMode::IndirectY),

            0xA2 => self.ldx(AddressingMode::Immediate),
            0xA6 => self.ldx(AddressingMode::ZeroPage),
            0xB6 => self.ldx(AddressingMode::ZeroPageY),
            0xAE => self.ldx(AddressingMode::Absolute),
            0xBE => self.ldx(AddressingMode::AbsoluteY),

            0xA0 => self.ldy(AddressingMode::Immediate),
            0xA4 => self.ldy(AddressingMode::ZeroPage),
            0xB4 => self.ldy(AddressingMode::ZeroPageX),
            0xAC => self.ldy(AddressingMode::Absolute),
            0xBC => self.ldy(AddressingMode::AbsoluteX),

            0x4A => self.lsr(AddressingMode::Accumulator),
            0x46 => self.lsr(AddressingMode::ZeroPage),
            0x56 => self.lsr(AddressingMode::ZeroPageX),
            0x4E => self.lsr(AddressingMode::Absolute),
            0x5E => self.lsr(AddressingMode::AbsoluteX),

            0xEA => self.nop(AddressingMode::Implied),

            0x09 => self.ora(AddressingMode::Immediate),
            0x05 => self.ora(AddressingMode::ZeroPage),
            0x15 => self.ora(AddressingMode::ZeroPageX),
            0x0D => self.ora(AddressingMode::Absolute),
            0x1D => self.ora(AddressingMode::AbsoluteX),
            0x19 => self.ora(AddressingMode::AbsoluteY),
            0x01 => self.ora(AddressingMode::IndirectX),
            0x11 => self.ora(AddressingMode::IndirectY),

            0x48 => self.pha(AddressingMode::Implied),
            0x08 => self.php(AddressingMode::Implied),
            0x68 => self.pla(AddressingMode::Implied),
            0x28 => self.plp(AddressingMode::Implied),

            0x2A => self.rol(AddressingMode::Accumulator),
            0x26 => self.rol(AddressingMode::ZeroPage),
            0x36 => self.rol(AddressingMode::ZeroPageX),
            0x2E => self.rol(AddressingMode::Absolute),
            0x3E => self.rol(AddressingMode::AbsoluteX),

            0x6A => self.ror(AddressingMode::Accumulator),
            0x66 => self.ror(AddressingMode::ZeroPage),
            0x76 => self.ror(AddressingMode::ZeroPageX),
            0x6E => self.ror(AddressingMode::Absolute),
            0x7E => self.ror(AddressingMode::AbsoluteX),

            0x40 => self.rti(AddressingMode::Implied),
            0x60 => self.rts(AddressingMode::Implied),

            0xE9 => self.sbc(AddressingMode::Immediate),
            0xE5 => self.sbc(AddressingMode::ZeroPage),
            0xF5 => self.sbc(AddressingMode::ZeroPageX),
            0xED => self.sbc(AddressingMode::Absolute),
            0xFD => self.sbc(AddressingMode::AbsoluteX),
            0xF9 => self.sbc(AddressingMode::AbsoluteY),
            0xE1 => self.sbc(AddressingMode::IndirectX),
            0xF1 => self.sbc(AddressingMode::IndirectY),

            0x38 => self.sec(AddressingMode::Implied),
            0xF8 => self.sed(AddressingMode::Implied),
            0x78 => self.sei(AddressingMode::Implied),

            0x85 => self.sta(AddressingMode::ZeroPage),
            0x95 => self.sta(AddressingMode::ZeroPageX),
            0x8D => self.sta(AddressingMode::Absolute),
            0x9D => self.sta(AddressingMode::AbsoluteX),
            0x99 => self.sta(AddressingMode::AbsoluteY),
            0x81 => self.sta(AddressingMode::IndirectX),
            0x91 => self.sta(AddressingMode::IndirectY),

            0x86 => self.stx(AddressingMode::ZeroPage),
            0x96 => self.stx(AddressingMode::ZeroPageY),
            0x8E => self.stx(AddressingMode::Absolute),

            0x84 => self.sty(AddressingMode::ZeroPage),
            0x94 => self.sty(AddressingMode::ZeroPageX),
            0x8C => self.sty(AddressingMode::Absolute),

            0xAA => self.tax(AddressingMode::Implied),
            0xA8 => self.tay(AddressingMode::Implied),
            0xBA => self.tsx(AddressingMode::Implied),
            0x8A => self.txa(AddressingMode::Implied),
            0x9A => self.txs(AddressingMode::Implied),
            0x98 => self.tya(AddressingMode::Implied),

            n => panic!("opcode {:X} not implemented", n),
        }
    }

    /// loads the byte at the program counter and advances the program counter.
    fn loadb_bump(&mut self) -> u8 {
        let opcode = self.readb(self.reg.pc);
        self.reg.pc += 1;
        opcode
    }

    /// loads the word at the program counter and advances the program counter.
    fn loadw_bump(&mut self) -> u16 {
        let lo = self.loadb_bump() as u16;
        let hi = self.loadb_bump() as u16;
        (hi << 8) | lo
    }

    fn readb(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize % 0x0800],
            0x2000..=0x3FFF => self.ppu.borrow_mut().read(addr % 0x08),
            0x4000..=0x4017 => self.apu[addr as usize % 0x0018],
            0x4018..=0x401F => 0,
            0x4020..=0xFFFF => self.cartridge.borrow().read(addr),
        }
    }

    fn readw_zp(&mut self, addr: u8) -> u16 {
        self.readb(addr as u16) as u16 | (self.readb((addr + 1) as u16) as u16) << 8
    }

    fn readw(&mut self, addr: u16) -> u16 {
        let lo = self.readb(addr) as u16;
        let hi = self.readb(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    fn writeb(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize % 0x0800] = val,
            0x2000..=0x3FFF => self.ppu.borrow_mut().write(addr % 0x08, val),
            0x4000..=0x4017 => self.apu[addr as usize % 0x0018] = val,
            0x4018..=0x401F => {}
            0x6000..=0x6003 => {}
            0x6004..=0x7FFF => {
                print!("{}", val as char);
            }
            0x4020..=0xFFFF => self.cartridge.borrow_mut().write(addr, val),
        }
    }

    fn set_zn(&mut self, res: u8) {
        self.reg.set_flag(Flag::Z, res == 0x00);
        self.reg.set_flag(Flag::N, res & 0x80 == 0x80);
    }
}

/// CPU opcodes
/// implemented as documented in https://www.masswerk.at/6502/6502_instruction_set.html
impl CPU {
    /// ADC  Add Memory to Accumulator with Carry
    ///  A + M + C -> A, C                N Z C I D V
    ///                                   + + + - - +
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  immidiate     ADC #oper     69    2     2
    ///  zeropage      ADC oper      65    2     3
    ///  zeropage,X    ADC oper,X    75    2     4
    ///  absolute      ADC oper      6D    3     4
    ///  absolute,X    ADC oper,X    7D    3     4*
    ///  absolute,Y    ADC oper,Y    79    3     4*
    ///  (indirect,X)  ADC (oper,X)  61    2     6
    ///  (indirect),Y  ADC (oper),Y  71    2     5*
    fn adc(&mut self, am: AddressingMode) -> u8 {
        let mem = am.load(self);
        let acc = self.reg.a;
        let res = mem as u16 + acc as u16;
        self.reg.set_flag(Flag::C, res > 0xFF);
        let res = res as u8;
        self.reg.set_flag(
            Flag::V,
            (acc ^ mem) & 0x80 == 0 && (acc ^ res) & 0x80 == 0x80,
        );
        self.set_zn(res as u8);
        self.reg.a = res;

        match am {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteX => 4,
            AddressingMode::AbsoluteY => 4,
            AddressingMode::IndirectX => 6,
            AddressingMode::IndirectY => 5,
            _ => unreachable!(),
        }
    }

    /// AND  AND Memory with Accumulator
    ///  A AND M -> A                     N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  immidiate     AND #oper     29    2     2
    ///  zeropage      AND oper      25    2     3
    ///  zeropage,X    AND oper,X    35    2     4
    ///  absolute      AND oper      2D    3     4
    ///  absolute,X    AND oper,X    3D    3     4*
    ///  absolute,Y    AND oper,Y    39    3     4*
    ///  (indirect,X)  AND (oper,X)  21    2     6
    ///  (indirect),Y  AND (oper),Y  31    2     5*
    fn and(&mut self, am: AddressingMode) -> u8 {
        let mem = am.load(self);
        let acc = self.reg.a;
        let res = mem & acc;
        self.set_zn(res as u8);
        self.reg.a = res;

        match am {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteX => 4,
            AddressingMode::AbsoluteY => 4,
            AddressingMode::IndirectX => 6,
            AddressingMode::IndirectY => 5,
            _ => unreachable!(),
        }
    }

    /// ASL  Shift Left One Bit (Memory or Accumulator)
    ///  C <- [76543210] <- 0             N Z C I D V
    ///                                   + + + - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  accumulator   ASL A         0A    1     2
    ///  zeropage      ASL oper      06    2     5
    ///  zeropage,X    ASL oper,X    16    2     6
    ///  absolute      ASL oper      0E    3     6
    ///  absolute,X    ASL oper,X    1E    3     7
    fn asl(&mut self, am: AddressingMode) -> u8 {
        let val = am.load(self);
        let res = (val as u16) << 1;
        am.debump(self);
        am.store(self, res as u8);
        self.reg.set_flag(Flag::C, res > 0xFF);
        self.set_zn(res as u8);

        match am {
            AddressingMode::Accumulator => 2,
            AddressingMode::ZeroPage => 5,
            AddressingMode::ZeroPageX => 6,
            AddressingMode::Absolute => 6,
            AddressingMode::AbsoluteX => 7,
            _ => unreachable!(),
        }
    }

    /// BCC  Branch on Carry Clear
    ///  branch on C = 0                  N Z C I D V
    ///                                   - - - - - -

    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  relative      BCC oper      90    2     2**
    fn bcc(&mut self, am: AddressingMode) -> u8 {
        self.branch_if(!self.reg.get_flag(Flag::C));

        match am {
            AddressingMode::Relative => 2,
            _ => unreachable!(),
        }
    }

    /// BCS  Branch on Carry Set
    ///  branch on C = 1                  N Z C I D V
    ///                                   - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  relative      BCS oper      B0    2     2**
    fn bcs(&mut self, am: AddressingMode) -> u8 {
        self.branch_if(self.reg.get_flag(Flag::C));

        match am {
            AddressingMode::Relative => 2,
            _ => unreachable!(),
        }
    }

    /// BEQ  Branch on Result Zero
    ///  branch on Z = 1                  N Z C I D V
    ///                                   - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  relative      BEQ oper      F0    2     2**
    fn beq(&mut self, am: AddressingMode) -> u8 {
        self.branch_if(self.reg.get_flag(Flag::Z));

        match am {
            AddressingMode::Relative => 2,
            _ => unreachable!(),
        }
    }

    /// BIT  Test Bits in Memory with Accumulator
    ///  bits 7 and 6 of operand are transfered to bit 7 and 6 of SR (N,V);
    ///  the zeroflag is set to the result of operand AND accumulator.
    ///
    ///  A AND M, M7 -> N, M6 -> V        N Z C I D V
    ///                                  M7 + - - - M6
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  zeropage      BIT oper      24    2     3
    ///  absolute      BIT oper      2C    3     4
    fn bit(&mut self, am: AddressingMode) -> u8 {
        let mem = am.load(self);
        self.reg.set_flag(Flag::N, 0x80 & mem == 0x80);
        self.reg.set_flag(Flag::V, 0x40 & mem == 0x40);
        self.reg.set_flag(Flag::Z, mem & self.reg.a == 0x00);

        match am {
            AddressingMode::ZeroPage => 3,
            AddressingMode::Absolute => 4,
            _ => unreachable!(),
        }
    }

    /// BMI  Branch on Result Minus
    ///  branch on N = 1                  N Z C I D V
    ///                                   - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  relative      BMI oper      30    2     2**
    fn bmi(&mut self, am: AddressingMode) -> u8 {
        self.branch_if(self.reg.get_flag(Flag::N));

        match am {
            AddressingMode::Relative => 2,
            _ => unreachable!(),
        }
    }

    /// BNE  Branch on Result not Zero
    ///  branch on Z = 0                  N Z C I D V
    ///                                   - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  relative      BNE oper      D0    2     2**
    fn bne(&mut self, am: AddressingMode) -> u8 {
        self.branch_if(!self.reg.get_flag(Flag::Z));

        match am {
            AddressingMode::Relative => 2,
            _ => unreachable!(),
        }
    }

    /// BPL  Branch on Result Plus
    ///  branch on N = 0                  N Z C I D V
    ///                                   - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  relative      BPL oper      10    2     2**
    fn bpl(&mut self, am: AddressingMode) -> u8 {
        self.branch_if(!self.reg.get_flag(Flag::N));

        match am {
            AddressingMode::Relative => 2,
            _ => unreachable!(),
        }
    }

    /// BRK  Force Break
    ///  interrupt,                       N Z C I D V
    ///  push PC+2, push SR               - - - 1 - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       BRK           00    1     7
    fn brk(&mut self, am: AddressingMode) -> u8 {
        let pc = self.reg.pc;
        self.pushw(pc + 1);
        let flags = self.reg.p;
        self.pushb(flags);
        self.reg.set_flag(Flag::I, true);
        self.reg.pc = self.readw(BRK_VECTOR);

        match am {
            AddressingMode::Implied => 7,
            _ => unreachable!(),
        }
    }

    /// BVC  Branch on Overflow Clear
    ///  branch on V = 0                  N Z C I D V
    ///                                   - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  relative      BVC oper      50    2     2**
    fn bvc(&mut self, am: AddressingMode) -> u8 {
        self.branch_if(!self.reg.get_flag(Flag::V));

        match am {
            AddressingMode::Relative => 2,
            _ => unreachable!(),
        }
    }

    /// BVS  Branch on Overflow Set
    ///  branch on V = 1                  N Z C I D V
    ///                                   - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  relative      BVC oper      70    2     2**
    fn bvs(&mut self, am: AddressingMode) -> u8 {
        self.branch_if(self.reg.get_flag(Flag::V));

        match am {
            AddressingMode::Relative => 2,
            _ => unreachable!(),
        }
    }

    /// CLC  Clear Carry Flag
    ///  0 -> C                           N Z C I D V
    ///                                   - - 0 - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       CLC           18    1     2
    fn clc(&mut self, am: AddressingMode) -> u8 {
        self.reg.set_flag(Flag::C, false);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// CLD  Clear Decimal Mode
    ///  0 -> D                           N Z C I D V
    ///                                   - - - - 0 -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       CLD           D8    1     2
    fn cld(&mut self, am: AddressingMode) -> u8 {
        self.reg.set_flag(Flag::D, false);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// CLI  Clear Interrupt Disable Bit
    ///  0 -> I                           N Z C I D V
    ///                                   - - - 0 - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       CLI           58    1     2
    fn cli(&mut self, am: AddressingMode) -> u8 {
        self.reg.set_flag(Flag::I, false);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// CLV  Clear Overflow Flag
    ///  0 -> V                           N Z C I D V
    ///                                   - - - - - 0
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       CLV           B8    1     2
    fn clv(&mut self, am: AddressingMode) -> u8 {
        self.reg.set_flag(Flag::V, false);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// CMP  Compare Memory with Accumulator
    ///  A - M                          N Z C I D V
    ///                                 + + + - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  immidiate     CMP #oper     C9    2     2
    ///  zeropage      CMP oper      C5    2     3
    ///  zeropage,X    CMP oper,X    D5    2     4
    ///  absolute      CMP oper      CD    3     4
    ///  absolute,X    CMP oper,X    DD    3     4*
    ///  absolute,Y    CMP oper,Y    D9    3     4*
    ///  (indirect,X)  CMP (oper,X)  C1    2     6
    ///  (indirect),Y  CMP (oper),Y  D1    2     5*
    fn cmp(&mut self, am: AddressingMode) -> u8 {
        let mem = am.load(self);
        self.compare(self.reg.a, mem);

        match am {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteX => 4,
            AddressingMode::AbsoluteY => 4,
            AddressingMode::IndirectX => 6,
            AddressingMode::IndirectY => 7,
            _ => unreachable!(),
        }
    }

    /// CPX  Compare Memory and Index X
    ///  X - M                            N Z C I D V
    ///                                   + + + - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  immidiate     CPX #oper     E0    2     2
    ///  zeropage      CPX oper      E4    2     3
    ///  absolute      CPX oper      EC    3     4
    fn cpx(&mut self, am: AddressingMode) -> u8 {
        let mem = am.load(self);
        self.compare(self.reg.x, mem);

        match am {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::Absolute => 4,
            _ => unreachable!(),
        }
    }

    /// CPY  Compare Memory and Index Y
    ///  Y - M                            N Z C I D V
    ///                                   + + + - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  immidiate     CPY #oper     C0    2     2
    ///  zeropage      CPY oper      C4    2     3
    ///  absolute      CPY oper      CC    3     4
    fn cpy(&mut self, am: AddressingMode) -> u8 {
        let mem = am.load(self);
        self.compare(self.reg.y, mem);

        match am {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::Absolute => 4,
            _ => unreachable!(),
        }
    }

    /// DEC  Decrement Memory by One
    ///  M - 1 -> M                       N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  zeropage      DEC oper      C6    2     5
    ///  zeropage,X    DEC oper,X    D6    2     6
    ///  absolute      DEC oper      CE    3     6
    ///  absolute,X    DEC oper,X    DE    3     7
    fn dec(&mut self, am: AddressingMode) -> u8 {
        let mem = am.load(self);
        let res = mem.wrapping_sub(1);
        am.debump(self);
        am.store(self, res);
        self.set_zn(res);

        match am {
            AddressingMode::ZeroPage => 5,
            AddressingMode::ZeroPageX => 6,
            AddressingMode::Absolute => 6,
            AddressingMode::AbsoluteX => 7,
            _ => unreachable!(),
        }
    }

    /// DEX  Decrement Index X by One
    ///  X - 1 -> X                       N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       DEC           CA    1     2
    fn dex(&mut self, am: AddressingMode) -> u8 {
        let x = self.reg.x;
        let res = x.wrapping_sub(1);
        self.reg.x = res;
        self.set_zn(res);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// DEY  Decrement Index Y by One
    ///  Y - 1 -> Y                       N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       DEC           88    1     2
    fn dey(&mut self, am: AddressingMode) -> u8 {
        let y = self.reg.y;
        let res = y.wrapping_sub(1);
        self.reg.y = res;
        self.set_zn(res);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// EOR  Exclusive-OR Memory with Accumulator
    ///  A EOR M -> A                     N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  immidiate     EOR #oper     49    2     2
    ///  zeropage      EOR oper      45    2     3
    ///  zeropage,X    EOR oper,X    55    2     4
    ///  absolute      EOR oper      4D    3     4
    ///  absolute,X    EOR oper,X    5D    3     4*
    ///  absolute,Y    EOR oper,Y    59    3     4*
    ///  (indirect,X)  EOR (oper,X)  41    2     6
    ///  (indirect),Y  EOR (oper),Y  51    2     5*
    fn eor(&mut self, am: AddressingMode) -> u8 {
        let mem = am.load(self);
        let acc = self.reg.a;
        let res = acc ^ mem;
        self.reg.a = res;
        self.set_zn(res);

        match am {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteX => 4,
            AddressingMode::AbsoluteY => 4,
            AddressingMode::IndirectX => 6,
            AddressingMode::IndirectY => 5,
            _ => unreachable!(),
        }
    }

    /// INC  Increment Memory by One
    ///  M + 1 -> M                       N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  zeropage      INC oper      E6    2     5
    ///  zeropage,X    INC oper,X    F6    2     6
    ///  absolute      INC oper      EE    3     6
    ///  absolute,X    INC oper,X    FE    3     7
    fn inc(&mut self, am: AddressingMode) -> u8 {
        let mem = am.load(self);
        let res = mem.wrapping_add(1);
        am.debump(self);
        am.store(self, res);
        self.set_zn(res);

        match am {
            AddressingMode::ZeroPage => 5,
            AddressingMode::ZeroPageX => 6,
            AddressingMode::Absolute => 6,
            AddressingMode::AbsoluteX => 7,
            _ => unreachable!(),
        }
    }

    /// INX  Increment Index X by One
    ///  X + 1 -> X                       N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       INX           E8    1     2
    fn inx(&mut self, am: AddressingMode) -> u8 {
        let reg = self.reg.x;
        let res = reg.wrapping_add(1);
        self.reg.x = res;
        self.set_zn(res);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// INY  Increment Index Y by One
    ///  Y + 1 -> Y                       N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       INY           C8    1     2
    fn iny(&mut self, am: AddressingMode) -> u8 {
        let reg = self.reg.y;
        let res = reg.wrapping_add(1);
        self.reg.y = res;
        self.set_zn(res);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// JMP  Jump to New Location
    ///  (PC+1) -> PCL                    N Z C I D V
    ///  (PC+2) -> PCH                    - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  absolute      JMP oper      4C    3     3
    ///  indirect      JMP (oper)    6C    3     5
    fn jmp(&mut self, am: AddressingMode) -> u8 {
        let res = self.loadw_bump();
        match am {
            AddressingMode::Absolute => self.reg.pc = res,
            AddressingMode::Indirect => {
                // blatant copy/paste from sprocketnes
                let lo = self.readb(res);
                let hi = self.readb((res & 0xff00) | ((res + 1) & 0x00ff));
                self.reg.pc = (hi as u16) << 8 | lo as u16;
            }
            _ => {}
        }

        match am {
            AddressingMode::Absolute => 3,
            AddressingMode::Indirect => 5,
            _ => unreachable!(),
        }
    }

    /// JSR  Jump to New Location Saving Return Address
    ///  push (PC+2),                     N Z C I D V
    ///  (PC+1) -> PCL                    - - - - - -
    ///  (PC+2) -> PCH
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  absolute      JSR oper      20    3     6
    fn jsr(&mut self, am: AddressingMode) -> u8 {
        let res = self.loadw_bump();
        let pc = self.reg.pc;
        self.pushw(pc - 1);
        self.reg.pc = res;

        match am {
            AddressingMode::Absolute => 6,
            _ => unreachable!(),
        }
    }

    /// LDA  Load Accumulator with Memory
    ///  M -> A                           N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  immidiate     LDA #oper     A9    2     2
    ///  zeropage      LDA oper      A5    2     3
    ///  zeropage,X    LDA oper,X    B5    2     4
    ///  absolute      LDA oper      AD    3     4
    ///  absolute,X    LDA oper,X    BD    3     4*
    ///  absolute,Y    LDA oper,Y    B9    3     4*
    ///  (indirect,X)  LDA (oper,X)  A1    2     6
    ///  (indirect),Y  LDA (oper),Y  B1    2     5*
    fn lda(&mut self, am: AddressingMode) -> u8 {
        let mem = am.load(self);
        self.reg.a = mem;
        self.set_zn(mem);

        match am {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteX => 4,
            AddressingMode::AbsoluteY => 4,
            AddressingMode::IndirectX => 6,
            AddressingMode::IndirectY => 5,
            _ => unreachable!(),
        }
    }

    /// LDX  Load Index X with Memory
    ///  M -> X                           N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  immidiate     LDX #oper     A2    2     2
    ///  zeropage      LDX oper      A6    2     3
    ///  zeropage,Y    LDX oper,Y    B6    2     4
    ///  absolute      LDX oper      AE    3     4
    ///  absolute,Y    LDX oper,Y    BE    3     4*
    fn ldx(&mut self, am: AddressingMode) -> u8 {
        let mem = am.load(self);
        self.reg.x = mem;
        self.set_zn(mem);

        match am {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageY => 4,
            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteY => 4,
            _ => unreachable!(),
        }
    }

    /// LDY  Load Index Y with Memory
    ///  M -> Y                           N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  immidiate     LDY #oper     A0    2     2
    ///  zeropage      LDY oper      A4    2     3
    ///  zeropage,X    LDY oper,X    B4    2     4
    ///  absolute      LDY oper      AC    3     4
    ///  absolute,X    LDY oper,X    BC    3     4*
    fn ldy(&mut self, am: AddressingMode) -> u8 {
        let mem = am.load(self);
        self.reg.y = mem;
        self.set_zn(mem);

        match am {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteX => 4,
            _ => unreachable!(),
        }
    }

    /// LSR  Shift One Bit Right (Memory or Accumulator)
    ///  0 -> [76543210] -> C             N Z C I D V
    ///                                   0 + + - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  accumulator   LSR A         4A    1     2
    ///  zeropage      LSR oper      46    2     5
    ///  zeropage,X    LSR oper,X    56    2     6
    ///  absolute      LSR oper      4E    3     6
    ///  absolute,X    LSR oper,X    5E    3     7
    fn lsr(&mut self, am: AddressingMode) -> u8 {
        let val = am.load(self);
        let c = val & 0x01;
        let res = val >> 1;
        am.debump(self);
        am.store(self, res as u8);
        self.reg.set_flag(Flag::C, c == 0x01);
        self.set_zn(res);

        match am {
            AddressingMode::Accumulator => 2,
            AddressingMode::ZeroPage => 5,
            AddressingMode::ZeroPageX => 6,
            AddressingMode::Absolute => 6,
            AddressingMode::AbsoluteX => 7,
            _ => unreachable!(),
        }
    }

    /// NOP  No Operation
    ///  ---                              N Z C I D V
    ///                                   - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       NOP           EA    1     2
    fn nop(&mut self, am: AddressingMode) -> u8 {
        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// ORA  OR Memory with Accumulator
    ///  A OR M -> A                      N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  immidiate     ORA #oper     09    2     2
    ///  zeropage      ORA oper      05    2     3
    ///  zeropage,X    ORA oper,X    15    2     4
    ///  absolute      ORA oper      0D    3     4
    ///  absolute,X    ORA oper,X    1D    3     4*
    ///  absolute,Y    ORA oper,Y    19    3     4*
    ///  (indirect,X)  ORA (oper,X)  01    2     6
    ///  (indirect),Y  ORA (oper),Y  11    2     5*
    fn ora(&mut self, am: AddressingMode) -> u8 {
        let mem = am.load(self);
        let acc = self.reg.a;
        let res = acc | mem;
        self.reg.a = res;
        self.set_zn(res);

        match am {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteX => 4,
            AddressingMode::AbsoluteY => 4,
            AddressingMode::IndirectX => 6,
            AddressingMode::IndirectY => 5,
            _ => unreachable!(),
        }
    }

    /// PHA  Push Accumulator on Stack
    ///  push A                           N Z C I D V
    ///                                   - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       PHA           48    1     3
    fn pha(&mut self, am: AddressingMode) -> u8 {
        let acc = self.reg.a;
        self.pushb(acc);

        match am {
            AddressingMode::Implied => 3,
            _ => unreachable!(),
        }
    }

    /// PHP  Push Processor Status on Stack
    ///  push SR                          N Z C I D V
    ///                                   - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       PHP           08    1     3
    fn php(&mut self, am: AddressingMode) -> u8 {
        let sr = self.reg.p | 0b0011_0000;
        self.pushb(sr);

        match am {
            AddressingMode::Implied => 3,
            _ => unreachable!(),
        }
    }

    /// PLA  Pull Accumulator from Stack
    ///  pull A                           N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       PLA           68    1     4
    fn pla(&mut self, am: AddressingMode) -> u8 {
        let val = self.popb();
        self.reg.a = val;
        self.set_zn(val);

        match am {
            AddressingMode::Implied => 4,
            _ => unreachable!(),
        }
    }

    /// PLP  Pull Processor Status from Stack
    ///  pull SR                          N Z C I D V
    ///                                   from stack
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       PLP           28    1     4
    fn plp(&mut self, am: AddressingMode) -> u8 {
        let val = self.popb();
        self.set_p(val);

        match am {
            AddressingMode::Implied => 4,
            _ => unreachable!(),
        }
    }

    /// ROL  Rotate One Bit Left (Memory or Accumulator)
    ///  C <- [76543210] <- C             N Z C I D V
    ///                                   + + + - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  accumulator   ROL A         2A    1     2
    ///  zeropage      ROL oper      26    2     5
    ///  zeropage,X    ROL oper,X    36    2     6
    ///  absolute      ROL oper      2E    3     6
    ///  absolute,X    ROL oper,X    3E    3     7
    fn rol(&mut self, am: AddressingMode) -> u8 {
        let val = am.load(self);
        let msb = val & 0x80;
        let c = self.reg.get_flag(Flag::C);
        let c = if c { 0x01 } else { 0x00 };
        let res = (val << 1) | c;
        am.debump(self);
        am.store(self, res);
        self.reg.set_flag(Flag::C, msb == 0x80);
        self.set_zn(res);

        match am {
            AddressingMode::Accumulator => 2,
            AddressingMode::ZeroPage => 5,
            AddressingMode::ZeroPageX => 6,
            AddressingMode::Absolute => 6,
            AddressingMode::AbsoluteX => 7,
            _ => unreachable!(),
        }
    }

    /// ROR  Rotate One Bit Right (Memory or Accumulator)
    ///  C -> [76543210] -> C             N Z C I D V
    ///                                   + + + - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  accumulator   ROR A         6A    1     2
    ///  zeropage      ROR oper      66    2     5
    ///  zeropage,X    ROR oper,X    76    2     6
    ///  absolute      ROR oper      6E    3     6
    ///  absolute,X    ROR oper,X    7E    3     7
    fn ror(&mut self, am: AddressingMode) -> u8 {
        let val = am.load(self);
        let lsb = val & 0x01;
        let c = self.reg.get_flag(Flag::C);
        let c = if c { 0x80 } else { 0x00 };
        let res = (val >> 1) | c;
        am.debump(self);
        am.store(self, res);
        self.reg.set_flag(Flag::C, lsb == 0x01);
        self.set_zn(res);

        match am {
            AddressingMode::Accumulator => 2,
            AddressingMode::ZeroPage => 5,
            AddressingMode::ZeroPageX => 6,
            AddressingMode::Absolute => 6,
            AddressingMode::AbsoluteX => 7,
            _ => unreachable!(),
        }
    }

    /// RTI  Return from Interrupt
    ///  pull SR, pull PC                 N Z C I D V
    ///                                   from stack
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       RTI           40    1     6
    fn rti(&mut self, am: AddressingMode) -> u8 {
        let sr = self.popb();
        self.set_p(sr);
        let pc = self.popw();
        self.reg.pc = pc;

        match am {
            AddressingMode::Implied => 6,
            _ => unreachable!(),
        }
    }

    /// RTS  Return from Subroutine
    ///  pull PC, PC+1 -> PC              N Z C I D V
    ///                                   - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       RTS           60    1     6
    fn rts(&mut self, am: AddressingMode) -> u8 {
        self.reg.pc = self.popw() + 1;

        match am {
            AddressingMode::Implied => 6,
            _ => unreachable!(),
        }
    }

    /// SBC  Subtract Memory from Accumulator with Borrow
    ///  A - M - C -> A                   N Z C I D V
    ///                                   + + + - - +
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  immidiate     SBC #oper     E9    2     2
    ///  zeropage      SBC oper      E5    2     3
    ///  zeropage,X    SBC oper,X    F5    2     4
    ///  absolute      SBC oper      ED    3     4
    ///  absolute,X    SBC oper,X    FD    3     4*
    ///  absolute,Y    SBC oper,Y    F9    3     4*
    ///  (indirect,X)  SBC (oper,X)  E1    2     6
    ///  (indirect),Y  SBC (oper),Y  F1    2     5*
    fn sbc(&mut self, am: AddressingMode) -> u8 {
        let mem = am.load(self);
        let acc = self.reg.a;
        let c = self.reg.get_flag(Flag::C);
        let c = if c { 0x00 } else { 0x01 };
        let res = (acc as u16).wrapping_sub(mem as u16).wrapping_sub(c as u16);
        self.reg.set_flag(Flag::C, res & 0x100 == 0);
        let res = res as u8;
        self.reg.set_flag(
            Flag::V,
            (acc ^ res) & 0x80 != 0 && (acc ^ mem) & 0x80 == 0x80,
        );
        self.set_zn(res);
        self.reg.a = res;

        match am {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteX => 4,
            AddressingMode::AbsoluteY => 4,
            AddressingMode::IndirectX => 6,
            AddressingMode::IndirectY => 5,
            _ => unreachable!(),
        }
    }

    /// SEC  Set Carry Flag
    ///  1 -> C                           N Z C I D V
    ///                                   - - 1 - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       SEC           38    1     2
    fn sec(&mut self, am: AddressingMode) -> u8 {
        self.reg.set_flag(Flag::C, true);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// SED  Set Decimal Flag
    ///  1 -> D                           N Z C I D V
    ///                                   - - - - 1 -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       SED           F8    1     2
    fn sed(&mut self, am: AddressingMode) -> u8 {
        self.reg.set_flag(Flag::D, true);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// SEI  Set Interrupt Disable Status
    ///  1 -> I                           N Z C I D V
    ///                                   - - - 1 - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       SEI           78    1     2
    fn sei(&mut self, am: AddressingMode) -> u8 {
        self.reg.set_flag(Flag::I, true);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// STA  Store Accumulator in Memory
    ///  A -> M                           N Z C I D V
    ///                                   - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  zeropage      STA oper      85    2     3
    ///  zeropage,X    STA oper,X    95    2     4
    ///  absolute      STA oper      8D    3     4
    ///  absolute,X    STA oper,X    9D    3     5
    ///  absolute,Y    STA oper,Y    99    3     5
    ///  (indirect,X)  STA (oper,X)  81    2     6
    ///  (indirect),Y  STA (oper),Y  91    2     6
    fn sta(&mut self, am: AddressingMode) -> u8 {
        let acc = self.reg.a;
        am.store(self, acc);

        match am {
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteX => 5,
            AddressingMode::AbsoluteY => 5,
            AddressingMode::IndirectX => 6,
            AddressingMode::IndirectY => 6,
            _ => unreachable!(),
        }
    }

    /// STX  Store Index X in Memory
    ///  X -> M                           N Z C I D V
    ///                                   - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  zeropage      STX oper      86    2     3
    ///  zeropage,Y    STX oper,Y    96    2     4
    ///  absolute      STX oper      8E    3     4
    fn stx(&mut self, am: AddressingMode) -> u8 {
        let reg = self.reg.x;
        am.store(self, reg);

        match am {
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageY => 4,
            AddressingMode::Absolute => 4,
            _ => unreachable!(),
        }
    }

    /// STY  Sore Index Y in Memory
    ///  Y -> M                           N Z C I D V
    ///                                   - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  zeropage      STY oper      84    2     3
    ///  zeropage,X    STY oper,X    94    2     4
    ///  absolute      STY oper      8C    3     4
    fn sty(&mut self, am: AddressingMode) -> u8 {
        let reg = self.reg.y;
        am.store(self, reg);

        match am {
            AddressingMode::ZeroPage => 2,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::Absolute => 4,
            _ => unreachable!(),
        }
    }

    /// TAX  Transfer Accumulator to Index X
    ///  A -> X                           N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       TAX           AA    1     2
    fn tax(&mut self, am: AddressingMode) -> u8 {
        let acc = self.reg.a;
        self.reg.x = acc;
        self.set_zn(acc);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// TAY  Transfer Accumulator to Index Y
    ///  A -> Y                           N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       TAY           A8    1     2
    fn tay(&mut self, am: AddressingMode) -> u8 {
        let acc = self.reg.a;
        self.reg.y = acc;
        self.set_zn(acc);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// TSX  Transfer Stack Pointer to Index X
    ///  SP -> X                          N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       TSX           BA    1     2
    fn tsx(&mut self, am: AddressingMode) -> u8 {
        let sp = self.reg.s;
        self.reg.x = sp;
        self.set_zn(sp);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// TXA  Transfer Index X to Accumulator
    ///  X -> A                           N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       TXA           8A    1     2
    fn txa(&mut self, am: AddressingMode) -> u8 {
        let reg = self.reg.x;
        self.reg.a = reg;
        self.set_zn(reg);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// TXS  Transfer Index X to Stack Register
    ///  X -> SP                          N Z C I D V
    ///                                   - - - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       TXS           9A    1     2
    fn txs(&mut self, am: AddressingMode) -> u8 {
        let reg = self.reg.x;
        self.reg.s = reg;
        self.set_zn(reg);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    /// TYA  Transfer Index Y to Accumulator
    ///  Y -> A                           N Z C I D V
    ///                                   + + - - - -
    ///
    ///  addressing    assembler    opc  bytes  cyles
    ///  --------------------------------------------
    ///  implied       TYA           98    1     2
    fn tya(&mut self, am: AddressingMode) -> u8 {
        let reg = self.reg.y;
        self.reg.a = reg;
        self.set_zn(reg);

        match am {
            AddressingMode::Implied => 2,
            _ => unreachable!(),
        }
    }

    // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
    fn set_p(&mut self, val: u8) {
        let b = self.reg.p & 0b0011_0000;
        self.reg.p = val & 0b1100_1111 | b;
    }

    fn popb(&mut self) -> u8 {
        self.reg.s = self.reg.s.wrapping_add(1);
        let sp = self.reg.s as u16;
        self.readb(0x100 | sp)
    }

    fn popw(&mut self) -> u16 {
        let lo = self.popb() as u16;
        let hi = self.popb() as u16;
        (hi << 8) | lo
    }

    fn pushb(&mut self, val: u8) {
        let sp = self.reg.s as u16;
        self.writeb(0x100 | sp, val);
        self.reg.s = self.reg.s.wrapping_sub(1);
    }

    fn pushw(&mut self, val: u16) {
        let hi = (val >> 8) as u8;
        let lo = (val & 0xFF) as u8;
        self.pushb(hi);
        self.pushb(lo);
    }

    /// performs a branch if the given condition is met.
    fn branch_if(&mut self, cond: bool) {
        let val = self.loadb_bump() as i8;
        if cond {
            self.reg.pc = (self.reg.pc as i32 + val as i32) as u16;
        }
    }

    /// performs x - y and set the appropiate flags.
    fn compare(&mut self, x: u8, y: u8) {
        let res = (x as u16).wrapping_sub(y as u16);
        self.set_zn(res as u8);
        self.reg.set_flag(Flag::C, x >= y);
    }
}

#[cfg(test)]
mod test {
    use crate::cartridge::Cartridge;
    use crate::cpu::CPU;
    use crate::ppu::PPU;
    use std::sync::{Rc, RefCell};

    #[test]
    fn test_read() {
        let mut data = [0; 0xFFFF];
        data[0xFFFD % 0xBFE0] = 0x00;
        data[0xFFFE % 0xBFE0] = 0x01;

        let cart = Cartridge::from_data(data.to_vec());
        let cart = Rc::new(RefCell::new(cart));
        let ppu = PPU::new(cart.clone());
        let ppu = Rc::new(RefCell::new(ppu));
        let mut cpu = CPU::new(cart, ppu.clone());

        let opcode = cpu.loadb_bump();
        assert_eq!(0x00, opcode);
        assert_eq!(0xFFFE, cpu.reg.pc);

        let opcode = cpu.loadb_bump();
        assert_eq!(0x01, opcode);
        assert_eq!(0xFFFF, cpu.reg.pc);
    }

    #[test]
    fn test_read_word() {
        let mut data = [0; 0xFFFF];
        data[0xFFFD % 0xBFE0] = 0x00;
        data[0xFFFE % 0xBFE0] = 0x01;

        let cart = Cartridge::from_data(data.to_vec());
        let cart = Rc::new(RefCell::new(cart));
        let ppu = PPU::new(cart.clone());
        let ppu = Rc::new(RefCell::new(ppu));
        let mut cpu = CPU::new(cart, ppu.clone());

        let word = cpu.readw(0xFFFD);
        assert_eq!(0x0100, word);
    }
}

#[test]
fn test_mirroring() {
    assert_eq!(2, 0x2002 % 0x08);
    assert_eq!(2, 0x200A % 0x08);
    assert_eq!(2, 0x2012 % 0x08);

    assert_eq!(0, 0x2000 % 0x08);
    assert_eq!(0, 0x2008 % 0x08);
    assert_eq!(0, 0x2010 % 0x08);
}
