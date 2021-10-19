use crate::parser::{Instruction, Label, Params};

use std::collections::HashMap;

use tracing::{info, instrument};

use color_eyre::{
    eyre::{eyre, WrapErr},
    Help, Report,
};

#[derive(Debug)]
struct State {
    running: bool,
    pc: usize,
    flag: bool,
    registers: Vec<usize>,
    program: Vec<Instruction>,
    tape: Option<Vec<usize>>,
    memory: Vec<usize>,
}
#[derive(Debug)]
pub struct TinyVM {
    params: Params,
    resolved_labels: HashMap<String, usize>,
    state: State,
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
            tape: None,
            memory: vec![0; 1 << params.word_size],
        };

        Self {
            params,
            resolved_labels,
            state,
        }
    }

    pub fn load_tape(&mut self, tape: Vec<usize>) {
        self.state.tape = Some(tape);
    }

    pub fn start(&mut self) {
        info!("TinyVM started");
        self.state.running = true;
    }

    pub fn stop(&mut self) {
        info!("TinyVM stopped");
        self.state.running = false;
    }

    pub fn step(&mut self) {
        todo!("Fetch-decode-execute")
    }

    pub fn run(&mut self) {
        self.start();
        while self.state.running {
            self.step();
        }
    }
}
