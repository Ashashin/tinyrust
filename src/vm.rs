use crate::parser::{Argument, Instruction, Params, Register};

use std::collections::HashMap;

use tracing::info;

use color_eyre::{
    eyre::{eyre, ErrReport},
    Report,
};

#[derive(Debug)]
struct State {
    running: bool,
    pc: usize,
    flag: bool,
    registers: Vec<usize>,
    program: Vec<Instruction>,
    tape: Vec<usize>,
    memory: Vec<usize>,
}
#[derive(Debug)]
pub struct TinyVM {
    params: Params,
    resolved_labels: HashMap<String, usize>,
    state: State,
    result: usize,
}

impl TinyVM {
    pub fn new(
        params: Params,
        program: Vec<Instruction>,
        resolved_labels: HashMap<String, usize>,
    ) -> Self {
        let state = State {
            running: false,
            pc: 0,
            flag: false,
            registers: vec![0; params.registers.into()],
            program,
            tape: vec![],
            memory: vec![],
        };

        Self {
            params,
            resolved_labels,
            state,
            result: 1,
        }
    }

    pub fn load_tape(&mut self, tape: Vec<usize>) {
        self.state.tape = tape;
    }

    pub fn start(&mut self) {
        info!("TinyVM started");
        self.state.running = true;
    }

    pub fn stop(&mut self) {
        info!("TinyVM stopped");
        self.state.running = false;
    }

    pub fn step(&mut self) -> Result<(), Report> {
        let instr = {
            match self.state.program.get(self.state.pc) {
                Some(instr) => instr.clone(),
                _ => {
                    return Err(eyre!(
                        "Segmentation fault: trying to access {}",
                        self.state.pc
                    ));
                }
            }
        };

        self.state.pc = self.execute(instr)?;

        Ok(())
    }

    pub fn display_memory(&self) {
        info!("memory: {:?}", self.state.memory);
    }

    pub fn display_registers(&self) {
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

    pub fn display_state(&self) {
        info!("flag: {}, pc: {}", self.state.flag, self.state.pc);
        self.display_memory();
        self.display_registers();
    }

    pub fn run(&mut self) -> Result<usize, Report> {
        self.start();
        while self.state.running {
            self.step()?;
        }
        Ok(self.result)
    }

    pub fn execute(&mut self, instr: Instruction) -> Result<usize, Report> {
        let mut next_pc = self.state.pc + 1;

        match instr {
            Instruction::Add(reg1, reg2, arg) => self.add(&reg1, &reg2, &arg),
            Instruction::Store(arg, reg) => self.store(&arg, &reg),
            Instruction::Mov(reg, arg) => self.mov(&reg, &arg),
            Instruction::Read(reg, arg) => self.read(&reg, &arg),
            Instruction::Jmp(arg) => next_pc = self.jmp(&arg),
            Instruction::CJmp(arg) => next_pc = self.cjmp(&arg),
            Instruction::Answer(arg) => self.answer(&arg),
            _ => unimplemented!("Unsupported instruction: {:?}", instr),
        }

        Ok(next_pc)
    }

    fn resolve(&self, arg: &Argument) -> usize {
        match arg {
            Argument::Imm(x) => (*x).try_into().unwrap(),
            Argument::Reg(reg) => self.state.registers[reg.index as usize],
            Argument::Label(ident) => self.resolved_labels[ident as &str],
        }
    }

    fn add(&mut self, reg1: &Register, reg2: &Register, arg: &Argument) {
        let msb_mask = 1 << (self.params.word_size - 1);
        let value_mask = (1 << self.params.word_size) - 1;

        let value1 = self.state.registers[reg2.index as usize];
        let value2 = self.resolve(arg);

        let result = (value1 + value2) & value_mask;
        let carry = (result & msb_mask) > 0;

        self.state.registers[reg1.index as usize] = result;
        self.state.flag = carry;
    }

    fn answer(&mut self, arg: &Argument) {
        let retval = self.resolve(arg);
        self.result = retval;
        self.stop();
    }

    fn jmp(&mut self, arg: &Argument) -> usize {
        self.resolve(arg)
    }

    fn cjmp(&mut self, arg: &Argument) -> usize {
        if !self.state.flag {
            self.state.pc + 1
        } else {
            self.jmp(arg)
        }
    }

    fn read(&mut self, reg: &Register, arg: &Argument) {
        let tape = self.resolve(arg);

        let has_tape = !self.state.tape.is_empty();

        let value = match tape {
            0 => {
                if !has_tape {
                    self.state.flag = true;
                    0
                } else {
                    self.state.flag = false;
                    self.state.tape.pop().unwrap()
                }
            }
            _ => {
                self.state.flag = true;
                0
            }
        };

        self.state.registers[reg.index as usize] = value;
    }

    fn mov(&mut self, reg: &Register, arg: &Argument) {
        let value = self.resolve(arg);
        self.state.registers[reg.index as usize] = value;
    }

    fn store(&mut self, arg: &Argument, reg: &Register) {
        // Store contents of register reg at the address arg
        let addr = self.resolve(arg);
        let value = self.state.registers[reg.index as usize];

        if self.state.memory.len() <= addr {
            self.state.memory.resize(addr + 1, 0);
        }

        self.state.memory[addr] = value;
    }
}
