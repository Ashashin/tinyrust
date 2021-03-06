use color_eyre::{eyre::eyre, Report};
use tracing::info;

use std::collections::HashMap;

use crate::parser::{Argument, Instruction, Params, Register};

/// Struct reprensenting the current state of the `TinyRAM` VM
#[derive(Debug)]
struct State {
    /// Indicates is the VM is currently running
    running: bool,
    /// The program counter of the VM
    pc: usize,
    /// Indicates if the flag is raised
    flag: bool,
    /// Represents the 8-bits registers of the VM
    registers: Vec<usize>,
    /// Represents the program run by the VM
    program: Vec<Instruction>,
    /// Reprensents the tape storing the inputs
    tape1: Vec<usize>,
    /// Reprensents the tape storing the inputs
    tape2: Vec<usize>,
    /// Represents the memory of the VM
    memory: Vec<u8>,
}

impl State {
    /// Init state
    fn init(program: Vec<Instruction>, register_nb: usize) -> Self {
        Self {
            running: false,
            pc: 0,
            flag: false,
            registers: vec![0; register_nb],
            program,
            tape1: vec![],
            tape2: vec![],
            memory: vec![],
        }
    }

    /// Allow the state to be processed by a callback
    fn process_state<F>(&self, func: &mut F)
    where
        F: FnMut(&[u8]),
    {
        func(&self.pc.to_be_bytes());

        func(&[self.flag as u8]);

        for el in &self.registers {
            func(&el.to_be_bytes());
        }
        for el in &self.memory {
            func(&el.to_be_bytes());
        }
    }

    /// Reset state
    fn reset(&mut self) {
        let reg = self.registers.len();

        self.running = false;
        self.pc = 0;
        self.flag = false;
        self.registers = vec![0; reg];
        self.tape1 = vec![];
        self.tape2 = vec![];
        self.memory = vec![];
    }
}

/// Structure representing the `TinyRAM` VM
#[derive(Debug)]
pub struct TinyVM {
    /// VM params
    params: Params,
    /// Hashmap of the labels and their positions
    resolved_labels: HashMap<String, usize>,
    /// State of the VM
    state: State,
    /// Output of the program run by the VM
    result: usize,
}

impl TinyVM {
    /// Create a new VM from a program
    pub fn new(
        params: Params,
        program: Vec<Instruction>,
        resolved_labels: HashMap<String, usize>,
    ) -> Self {
        let state = State::init(program, params.registers.into());

        Self {
            params,
            resolved_labels,
            state,
            result: 1,
        }
    }

    /// Load the input tapes into the VM
    pub fn load_tapes(&mut self, tape: (Vec<usize>, Vec<usize>)) {
        self.state.tape1 = tape.0;
        self.state.tape2 = tape.1;
    }

    /// Read the next word in the primary input tape
    fn read_primary_tape(&mut self) -> usize {
        self.state.tape1.pop().unwrap_or(0)
    }

    /// Read the next word in the secondary input tape
    fn read_secondary_tape(&mut self) -> usize {
        self.state.tape2.pop().unwrap_or(0)
    }

    /// Launch the VM
    fn start(&mut self) {
        info!("TinyVM started");
        self.state.running = true;
    }

    /// Halt the VM
    fn stop(&mut self) {
        info!("TinyVM stopped");
        self.state.running = false;
    }

    /// Run the current instruction marked by the pc
    fn step(&mut self) -> Result<(), Report> {
        let instr = {
            match self.state.program.get(self.state.pc) {
                Some(instr) => instr.clone(),
                _ => Self::segfault(),
            }
        };

        self.state.pc = self.execute(instr)?;

        Ok(())
    }

    /// Print the current state of the memory
    fn display_memory(&self) {
        info!("memory: {:?}", self.state.memory);
    }

    /// Print the current state of the registers
    fn display_registers(&self) {
        let reg_data: String = self
            .state
            .registers
            .iter()
            .enumerate()
            .map(|(i, val)| format!("r{}: {}", i, val))
            .collect::<Vec<String>>()
            .join(", ");

        info!("registers: ({})", reg_data);
    }

    /// Print the current state of the VM
    fn display_state(&self) {
        info!("flag: {}, pc: {}", self.state.flag, self.state.pc);
        self.display_registers();
        self.display_memory();
    }

    /// Return the instructions from the current program loaded in the VM
    pub fn instructions(&self) -> Vec<Instruction> {
        self.state.program.clone()
    }

    /// Run the program loaded in the VM
    fn run<F>(&mut self, mut callback: F) -> Result<usize, Report>
    where
        F: FnMut(&[u8]),
    {
        self.start();
        while self.state.running {
            self.step()?;
            self.state.process_state(&mut callback);
        }

        Ok(self.result)
    }

    /// Run the VM with the selected input
    pub fn run_vm(&mut self, input: (Vec<usize>, Vec<usize>)) -> Result<usize, Report> {
        self.run_vm_with_callback(input, |_: &[u8]| {})
    }

    /// Run the VM with a callback and the selected input
    pub fn run_vm_with_callback<F>(
        &mut self,
        input: (Vec<usize>, Vec<usize>),
        callback: F,
    ) -> Result<usize, Report>
    where
        F: FnMut(&[u8]),
    {
        self.load_tapes(input);

        info!("??? All good to go! ???");
        match self.run(callback)? {
            0 => {
                info!("??? TinyVM terminated without error ???");
                self.display_state();

                Ok(self.output())
            }
            x => Err(eyre!("???? Program terminated with error code {} ????", x)),
        }
    }

    /// Displays the output of the program
    pub fn output(&self) -> usize {
        let val: [u8; 8] = <[u8; 8]>::try_from(&self.state.memory[0..8]).unwrap();

        usize::from_le_bytes(val)
    }

    /// Reset the state of the VM to initial state
    pub fn reset_state(&mut self) {
        self.state.reset();
    }

    /// Read value from the designated register
    fn read_reg(&self, reg: &Register) -> usize {
        self.state.registers[reg.index as usize]
    }

    /// Write value from the designated register
    fn write_reg(&mut self, reg: &Register, val: usize) {
        self.state.registers[reg.index as usize] = val;
    }

    /// Convert unsigned to signed
    fn to_signed(x: u64) -> i64 {
        unsafe { std::mem::transmute::<u64, i64>(x) }
    }

    /// Convert signed to unsigned
    fn to_unsigned(x: i64) -> u64 {
        unsafe { std::mem::transmute::<i64, u64>(x) }
    }

    /// Execute the instruction at the current pc
    fn execute(&mut self, instr: Instruction) -> Result<usize, Report> {
        let mut next_pc = self.state.pc + 1;

        match instr {
            // Bit operations
            Instruction::And(reg1, reg2, arg) => self.and(&reg1, &reg2, &arg),
            Instruction::Or(reg1, reg2, arg) => self.or(&reg1, &reg2, &arg),
            Instruction::Xor(reg1, reg2, arg) => self.xor(&reg1, &reg2, &arg),
            Instruction::Not(reg, arg) => self.not(&reg, &arg),

            // Integer operations
            Instruction::Add(reg1, reg2, arg) => self.add(&reg1, &reg2, &arg),
            Instruction::Sub(reg1, reg2, arg) => self.sub(&reg1, &reg2, &arg),
            Instruction::MulL(reg1, reg2, arg) => self.mull(&reg1, &reg2, &arg),
            Instruction::UMulH(_reg1, _reg2, _arg) => unimplemented!("UMulH"),
            Instruction::SMulH(_reg1, _reg2, _arg) => unimplemented!("SMulH"),
            Instruction::UDiv(reg1, reg2, arg) => self.udiv(&reg1, &reg2, &arg),
            Instruction::UMod(reg1, reg2, arg) => self.umod(&reg1, &reg2, &arg),

            // Shift operations
            Instruction::Shl(reg1, reg2, arg) => self.shl(&reg1, &reg2, &arg),
            Instruction::Shr(reg1, reg2, arg) => self.shr(&reg1, &reg2, &arg),

            // Compare operations
            Instruction::CmpE(reg, arg) => self.cmpe(&reg, &arg),
            Instruction::CmpA(reg, arg) => self.cmpa(&reg, &arg),
            Instruction::CmpAE(reg, arg) => self.cmpae(&reg, &arg),
            Instruction::CmpG(reg, arg) => self.cmpg(&reg, &arg),
            Instruction::CmpGE(reg, arg) => self.cmpge(&reg, &arg),

            // Move operations
            Instruction::Mov(reg, arg) => self.mov(&reg, &arg),
            Instruction::CMov(reg, arg) => self.cmov(&reg, &arg),

            // Jump operations
            Instruction::Jmp(arg) => next_pc = self.jmp(&arg),
            Instruction::CJmp(arg) => next_pc = self.cjmp(&arg),
            Instruction::CnJmp(arg) => next_pc = self.cnjmp(&arg),

            // Memory operations
            Instruction::StoreB(arg, reg) => self.store_b(&arg, &reg),
            Instruction::StoreW(arg, reg) => self.store_w(&arg, &reg),
            Instruction::LoadB(reg, arg) => self.load_b(&reg, &arg),
            Instruction::LoadW(reg, arg) => self.load_w(&reg, &arg),

            // Input operation
            Instruction::Read(reg, arg) => self.read(&reg, &arg),

            // Answer operation
            Instruction::Answer(arg) => {
                next_pc -= 1;
                self.answer(&arg);
            }
        }

        Ok(next_pc)
    }

    /// Resolve argument as label, register or value
    fn resolve(&self, arg: &Argument) -> usize {
        match arg {
            Argument::Imm(x) => Self::to_unsigned(*x) as usize,
            Argument::Reg(reg) => self.read_reg(reg),
            Argument::Label(ident) => self.resolved_labels[ident as &str],
        }
    }

    /// Defines the segfault instruction
    const fn segfault() -> Instruction {
        Instruction::Answer(Argument::Imm(1))
    }

    /// Defines the `TinyRAM` "and" instruction
    fn and(&mut self, reg1: &Register, reg2: &Register, arg: &Argument) {
        let value1 = self.read_reg(reg2);
        let value2 = self.resolve(arg);

        let result = value1 & value2;
        let zero = result == 0;

        self.write_reg(reg1, result);
        self.state.flag = zero;
    }

    /// Defines the `TinyRAM` "or" instruction
    fn or(&mut self, reg1: &Register, reg2: &Register, arg: &Argument) {
        let value1 = self.read_reg(reg2);
        let value2 = self.resolve(arg);

        let result = value1 | value2;
        let zero = result == 0;

        self.write_reg(reg1, result);
        self.state.flag = zero;
    }

    /// Defines the `TinyRAM` "xor" instruction
    fn xor(&mut self, reg1: &Register, reg2: &Register, arg: &Argument) {
        let value1 = self.read_reg(reg2);
        let value2 = self.resolve(arg);

        let result = value1 ^ value2;
        let zero = result == 0;

        self.write_reg(reg1, result);
        self.state.flag = zero;
    }

    /// Defines the `TinyRAM` "not" instruction
    fn not(&mut self, reg: &Register, arg: &Argument) {
        let value = self.resolve(arg);

        let result = !value;
        let zero = result == 0;

        self.write_reg(reg, result);
        self.state.flag = zero;
    }

    /// Defines the `TinyRAM` "add" instruction
    fn add(&mut self, reg1: &Register, reg2: &Register, arg: &Argument) {
        let msb_mask = 1 << (self.params.word_size - 1);

        // HOTFIX: 2^64 will overflow otherwise
        let value_mask = if self.params.word_size == 64 {
            usize::MAX
        } else {
            (1 << self.params.word_size) - 1
        };

        let value1 = self.read_reg(reg2);
        let value2 = self.resolve(arg);

        let result = (value1 + value2) & value_mask;
        let carry = (result & msb_mask) > 0;

        self.write_reg(reg1, result);
        self.state.flag = carry;
    }

    /// Defines the `TinyRAM` "sub" instruction
    fn sub(&mut self, reg1: &Register, reg2: &Register, arg: &Argument) {
        let msb_mask = 1 << (self.params.word_size - 1);

        // HOTFIX: 2^64 will overflow otherwise
        let value_mask = if self.params.word_size == 64 {
            usize::MAX
        } else {
            (1 << self.params.word_size) - 1
        };

        let value1 = self.read_reg(reg2);
        let value2 = self.resolve(arg);

        let result = (value_mask - value2 + value1 + 1) & value_mask;
        let carry = (result & msb_mask) > 0;

        self.write_reg(reg1, result);
        self.state.flag = !carry;
    }

    /// Defines the `TinyRAM` "mull" instruction
    fn mull(&mut self, reg1: &Register, reg2: &Register, arg: &Argument) {
        // HOTFIX: 2^64 will overflow otherwise
        let value_mask = if self.params.word_size == 64 {
            usize::MAX
        } else {
            (1 << self.params.word_size) - 1
        };

        let value1 = self.read_reg(reg2);
        let value2 = self.resolve(arg);

        let result = value1 * value2;
        let carry = result > value_mask;
        let result = result & value_mask;

        self.write_reg(reg1, result);
        self.state.flag = carry;
    }

    /// Defines the `TinyRAM` "udiv" instruction
    fn udiv(&mut self, reg1: &Register, reg2: &Register, arg: &Argument) {
        // HOTFIX: 2^64 will overflow otherwise
        let value_mask = if self.params.word_size == 64 {
            usize::MAX
        } else {
            (1 << self.params.word_size) - 1
        };

        let value1 = self.resolve(arg);

        let (result, flag) = if value1 == 0 {
            (0, true)
        } else {
            let value2 = self.read_reg(reg2);
            ((value2 / value1) & value_mask, false)
        };

        self.write_reg(reg1, result);
        self.state.flag = flag;
    }

    /// Defines the `TinyRAM` "umod" instruction
    fn umod(&mut self, reg1: &Register, reg2: &Register, arg: &Argument) {
        // HOTFIX: 2^64 will overflow otherwise
        let value_mask = if self.params.word_size == 64 {
            usize::MAX
        } else {
            (1 << self.params.word_size) - 1
        };

        let value1 = self.resolve(arg);

        let (result, flag) = if value1 == 0 {
            (0, true)
        } else {
            let value2 = self.read_reg(reg2);
            ((value2 % value1) & value_mask, false)
        };

        self.write_reg(reg1, result);
        self.state.flag = flag;
    }

    /// Defines the `TinyRAM` "shl" instruction
    fn shl(&mut self, reg1: &Register, reg2: &Register, arg: &Argument) {
        let value1 = self.resolve(arg);
        let value2 = self.read_reg(reg2);

        // HOTFIX: 2^64 will overflow otherwise
        let value_mask = if self.params.word_size == 64 {
            usize::MAX
        } else {
            (1 << self.params.word_size) - 1
        };

        let msb_mask = 1 << (self.params.word_size - 1);

        let result = (value2 << value1) & value_mask;
        let carry = (result & msb_mask) > 0;

        self.write_reg(reg1, result);
        self.state.flag = carry;
    }

    /// Defines the `TinyRAM` "shr" instruction
    fn shr(&mut self, reg1: &Register, reg2: &Register, arg: &Argument) {
        let value1 = self.resolve(arg);
        let value2 = self.read_reg(reg2);

        // HOTFIX: 2^64 will overflow otherwise
        let value_mask = if self.params.word_size == 64 {
            usize::MAX
        } else {
            (1 << self.params.word_size) - 1
        };

        let lsb_mask = 1;

        let result = (value2 >> value1) & value_mask;
        let carry = (result & lsb_mask) > 0;

        self.write_reg(reg1, result);
        self.state.flag = carry;
    }

    /// Defines the `TinyRAM` "cmpe" instruction
    fn cmpe(&mut self, reg: &Register, arg: &Argument) {
        let value1 = self.resolve(arg);
        let value2 = self.read_reg(reg);

        let equal = value1 == value2;
        self.state.flag = equal;
    }

    /// Defines the `TinyRAM` "cmpa" instruction
    fn cmpa(&mut self, reg: &Register, arg: &Argument) {
        let value1 = self.resolve(arg);
        let value2 = self.read_reg(reg);

        let above = value1 < value2;
        self.state.flag = above;
    }

    /// Defines the `TinyRAM` "cmpae" instruction
    fn cmpae(&mut self, reg: &Register, arg: &Argument) {
        let value1 = self.resolve(arg);
        let value2 = self.read_reg(reg);

        let above = value1 <= value2;
        self.state.flag = above;
    }

    /// Defines the `TinyRAM` "cmpg" instruction
    fn cmpg(&mut self, reg: &Register, arg: &Argument) {
        let value1 = Self::to_signed(self.resolve(arg) as u64);
        let value2 = Self::to_signed(self.read_reg(reg) as u64);

        let above = value1 < value2;
        self.state.flag = above;
    }

    /// Defines the `TinyRAM` "cmpge" instruction
    fn cmpge(&mut self, reg: &Register, arg: &Argument) {
        let value1 = Self::to_signed(self.resolve(arg) as u64);
        let value2 = Self::to_signed(self.read_reg(reg) as u64);

        let above = value1 <= value2;
        self.state.flag = above;
    }

    /// Defines the `TinyRAM` "amswer" instruction
    fn answer(&mut self, arg: &Argument) {
        let retval = self.resolve(arg);
        self.result = retval;
        self.stop();
    }

    /// Defines the `TinyRAM` "jmp" instruction
    fn jmp(&mut self, arg: &Argument) -> usize {
        self.resolve(arg)
    }

    /// Defines the `TinyRAM` "cjmp" instruction
    fn cjmp(&mut self, arg: &Argument) -> usize {
        if self.state.flag {
            self.jmp(arg)
        } else {
            self.state.pc + 1
        }
    }

    /// Defines the `TinyRAM` "cnjmp" instruction
    fn cnjmp(&mut self, arg: &Argument) -> usize {
        if self.state.flag {
            self.state.pc + 1
        } else {
            self.jmp(arg)
        }
    }

    /// Defines the `TinyRAM` "read" instruction
    fn read(&mut self, reg: &Register, arg: &Argument) {
        let tape = self.resolve(arg);
        let has_tape = (!self.state.tape1.is_empty(), !self.state.tape2.is_empty());

        let value = match (tape, has_tape) {
            (0, (true, _)) => {
                self.state.flag = false;
                self.read_primary_tape()
            }
            (1, (_, true)) => {
                self.state.flag = false;
                self.read_secondary_tape()
            }
            _ => {
                self.state.flag = true;
                0
            }
        };

        self.write_reg(reg, value);
    }

    /// Defines the `TinyRAM` "mov" instruction
    fn mov(&mut self, reg: &Register, arg: &Argument) {
        let value = self.resolve(arg);
        self.write_reg(reg, value);
    }

    /// Defines the `TinyRAM` "cmov" instruction
    fn cmov(&mut self, reg: &Register, arg: &Argument) {
        if self.state.flag {
            self.mov(reg, arg);
        }
    }

    /// Defines the `TinyRAM` "store.b" instruction
    fn store_b(&mut self, arg: &Argument, reg: &Register) {
        let addr = self.resolve(arg);
        let value = self.read_reg(reg);

        // HOTFIX: 2^64 will overflow otherwise
        let value_mask = if self.params.word_size == 64 {
            usize::MAX
        } else {
            (1 << self.params.word_size) - 1
        };

        let result = value & value_mask;

        if self.state.memory.len() <= addr {
            self.state.memory.resize(addr + 1, 0);
        }

        self.state.memory[addr] = result as u8;
    }

    /// Defines the `TinyRAM` "store.w" instruction
    fn store_w(&mut self, arg: &Argument, reg: &Register) {
        let addr = self.resolve(arg);
        let value = self.read_reg(reg);

        if self.state.memory.len() < addr + 8 {
            self.state.memory.resize(addr + 8, 0)
        }

        self.state
            .memory
            .splice(addr..(addr + 8), value.to_le_bytes());
    }

    /// Defines the `TinyRAM` "load.b" instruction
    fn load_b(&mut self, reg: &Register, arg: &Argument) {
        let addr = self.resolve(arg);
        let val = self.state.memory[addr] as usize;

        self.write_reg(reg, val as usize);
    }

    /// Defines the `TinyRAM` "load.w" instruction
    fn load_w(&mut self, reg: &Register, arg: &Argument) {
        let addr = self.resolve(arg);
        let val: [u8; 8] = <[u8; 8]>::try_from(&self.state.memory[addr..(addr + 8)]).unwrap();

        self.write_reg(reg, usize::from_le_bytes(val));
    }
}
