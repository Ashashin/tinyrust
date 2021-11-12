use color_eyre::{
    eyre::{eyre, WrapErr},
    Help, Report,
};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::info;

use std::{
    collections::HashMap,
    fmt::Debug,
    fs::File,
    io::{self, BufRead},
    path::Path,
};

use crate::vm::TinyVM;

/// Defines a register
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Register {
    /// Register index
    pub index: u16,
}

/// Defines a label
#[derive(Debug)]
pub struct Label {
    /// Label identifier
    ident: String,
    /// Address of the label
    address: usize,
    /// Line referenced by the label
    line: usize,
}

#[derive(Debug, Copy, Clone)]
pub enum ArchType {
    Harvard,
    VonNeumann,
    Unknown,
}

/// `TinyRAM` VM params
#[derive(Debug, Copy, Clone)]
pub struct Params {
    /// Version of the tinyRAM spec
    version: f32,
    /// W parameter: Word size
    pub word_size: u16,
    /// K parameter: Number of registers
    pub registers: u16,
    /// M parameter: chitecture type of the VM
    pub arch: ArchType,
}

/// Enum encompassing all value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Argument {
    /// Value
    Imm(i64),
    /// Register
    Reg(Register),
    /// Label
    Label(String),
}

/// Enum listing all instructions of the `TinyRAM` VM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    And(Register, Register, Argument),
    Or(Register, Register, Argument),
    Xor(Register, Register, Argument),
    Not(Register, Argument),
    Add(Register, Register, Argument),
    Sub(Register, Register, Argument),
    MulL(Register, Register, Argument),
    UMulH(Register, Register, Argument),
    SMulH(Register, Register, Argument),
    UDiv(Register, Register, Argument),
    UMod(Register, Register, Argument),
    Shl(Register, Register, Argument),
    Shr(Register, Register, Argument),

    CmpE(Register, Argument),
    CmpA(Register, Argument),
    CmpAE(Register, Argument),
    CmpG(Register, Argument),
    CmpGE(Register, Argument),

    Mov(Register, Argument),
    CMov(Register, Argument),

    Jmp(Argument),
    CJmp(Argument),
    CnJmp(Argument),

    StoreB(Argument, Register),
    StoreW(Argument, Register),
    LoadB(Register, Argument),
    LoadW(Register, Argument),
    Read(Register, Argument),

    Answer(Argument),
}

/// Parser form the `TinyRAM` programs
pub struct Parser;

impl Parser {
    /// Parse tapefile into a valide tape
    pub fn load_tape_file<P>(filename: &P) -> Result<Vec<usize>, Report>
    where
        P: AsRef<Path> + Debug,
    {
        info!("Loading tape from {:?}", filename);

        let lines = Self::read_lines(filename)?;
        let mut tape = vec![];
        for (_idx, line) in lines.enumerate() {
            let line = line.unwrap();
            let value = line.parse::<u64>()? as usize;
            tape.push(value);
        }

        info!("Tape loaded with {} entries", tape.len());

        Ok(tape)
    }

    /// Parse `TinyRAM` program into a `TinyRAM` VM
    pub fn load_program<P>(filename: &P) -> Result<TinyVM, Report>
    where
        P: AsRef<Path> + Debug,
    {
        info!("Processing file {:?}", filename.as_ref());
        let mut lines = Self::read_lines(filename)?;

        // Check header
        let first_line = lines.next().unwrap().unwrap();
        let params = Self::read_params(&first_line)
            .wrap_err_with(|| "Line 1: Incorrect parameters")
            .with_suggestion(|| {
                "The first line should be '; TinyRAM V=[version] W=[wordsize] K=[registers]'"
            })?;

        Self::check_params(params)?;

        // Parsing
        let mut instructions = vec![];
        let mut labels = vec![];

        for (idx, line) in lines.enumerate() {
            let line = line.unwrap();
            let line = line.trim();

            if line.is_empty()
                || Self::parse_comment(line).is_some()
                || Self::parse_whitespace(line).is_some()
            {
                continue;
            } else if let Some(instr) = Self::parse_instruction(line) {
                instructions.push(instr);
                continue;
            } else if let Some(label) = Self::parse_label(line) {
                labels.push(Label {
                    ident: label,
                    address: instructions.len(),
                    line: idx + 2,
                });
                continue;
            }

            return Err(eyre!("Line {}: Invalid content '{}'", idx + 2, line));
        }

        // Resolution
        let resolved_labels = Self::check_and_resolve_labels(&labels)?;
        Self::check_instructions(params, &instructions, &resolved_labels)?;

        Ok(TinyVM::new(params, instructions, resolved_labels))
    }

    /// Check if `TinyRAM` params are valid
    #[allow(clippy::float_cmp)]
    fn check_params(params: Params) -> Result<(), Report> {
        if params.version != 2.0 {
            return Err(eyre!("Unsupported version: {}", params.version));
        } else if params.word_size % 8 != 0 && params.word_size.is_power_of_two() {
            return Err(eyre!(
                "Word size should be a power of two and divisible by 8"
            ));
        }

        match params.arch {
            ArchType::Harvard => Ok(()),
            ArchType::VonNeumann => Err(eyre!("Tinyrust only supports Harvard architecture (hv)")),
            ArchType::Unknown => Err(eyre!("Unknown VM architecture")),
        }
    }

    /// Check if parsed instructions are valid
    fn check_instructions(
        params: Params,
        instructions: &[Instruction],
        resolved_labels: &HashMap<String, usize>,
    ) -> Result<(), Report> {
        info!("Checking instructions");

        let check_reg = |reg: &Register| {
            if reg.index >= params.registers {
                Err(eyre!("Register 'r{}' does not exist", reg.index))
            } else {
                Ok(())
            }
        };

        let check_arg = |arg: &Argument| match arg {
            Argument::Reg(reg) => check_reg(reg),
            Argument::Label(ident) => {
                if resolved_labels.contains_key(ident as &str) {
                    Ok(())
                } else {
                    Err(eyre!("Undefined label '{}'", ident))
                }
            }
            Argument::Imm(_) => Ok(()),
        };

        for instr in instructions {
            match instr {
                Instruction::Jmp(arg)
                | Instruction::CJmp(arg)
                | Instruction::CnJmp(arg)
                | Instruction::Answer(arg) => {
                    check_arg(arg)?;
                }
                Instruction::Not(reg, arg)
                | Instruction::Mov(reg, arg)
                | Instruction::CMov(reg, arg)
                | Instruction::LoadB(reg, arg)
                | Instruction::LoadW(reg, arg)
                | Instruction::Read(reg, arg)
                | Instruction::CmpE(reg, arg)
                | Instruction::CmpGE(reg, arg)
                | Instruction::CmpG(reg, arg)
                | Instruction::CmpA(reg, arg)
                | Instruction::CmpAE(reg, arg)
                | Instruction::StoreB(arg, reg)
                | Instruction::StoreW(arg, reg) => {
                    check_reg(reg)?;
                    check_arg(arg)?;
                }
                Instruction::And(reg1, reg2, arg)
                | Instruction::Or(reg1, reg2, arg)
                | Instruction::Xor(reg1, reg2, arg)
                | Instruction::Add(reg1, reg2, arg)
                | Instruction::Sub(reg1, reg2, arg)
                | Instruction::MulL(reg1, reg2, arg)
                | Instruction::UMulH(reg1, reg2, arg)
                | Instruction::SMulH(reg1, reg2, arg)
                | Instruction::UDiv(reg1, reg2, arg)
                | Instruction::UMod(reg1, reg2, arg)
                | Instruction::Shl(reg1, reg2, arg)
                | Instruction::Shr(reg1, reg2, arg) => {
                    check_reg(reg1)?;
                    check_reg(reg2)?;
                    check_arg(arg)?;
                }
            }
        }
        Ok(())
    }

    /// Check if labels are valid and resolve the labels
    fn check_and_resolve_labels(labels: &[Label]) -> Result<HashMap<String, usize>, Report> {
        info!("Resolving labels");

        let mut hashmap = HashMap::new();
        for label in labels {
            let duplicate = hashmap.insert(label.ident.clone(), label.address);
            if duplicate.is_some() {
                return Err(eyre!(
                    "Line {}: Duplicate label: '{}'",
                    label.line,
                    label.ident,
                ));
            }
        }
        Ok(hashmap)
    }

    /// Read the `TimyRAM` VM params from the first line of the program file
    fn read_params(first_line: &str) -> Result<Params, Report> {
        let parts: Vec<_> = first_line.split_whitespace().collect();

        if parts.len() != 6 {
            return Err(eyre!("First line should state machine parameters"));
        } else if parts[0] != ";" {
            return Err(eyre!("First line should be a comment (start by ';')"));
        } else if parts[1] != "TinyRAM" {
            return Err(eyre!("Magic string 'TinyRAM' is missing"));
        }

        let version = parts[2][2..].parse::<f32>()?;
        let word_size = parts[4][2..].parse::<u16>()?;
        let registers = parts[5][2..].parse::<u16>()?;

        let arch = match &parts[3][2..] {
            "hv" => ArchType::Harvard,
            "vn" => ArchType::VonNeumann,
            _ => ArchType::Unknown,
        };

        Ok(Params {
            version,
            word_size,
            registers,
            arch,
        })
    }

    /// Parse instruction from the current line
    fn parse_instruction(line: &str) -> Option<Instruction> {
        let mut parts: Vec<_> = line.split_whitespace().collect();

        // Discard comments if any
        for (idx, part) in parts.iter().enumerate() {
            if Self::parse_comment(part).is_some() {
                parts.truncate(idx);
                break;
            }
        }

        // Discard empty line
        if parts.is_empty() {
            return None;
        }

        let mut operands = vec![];
        let opcode = parts[0];
        let nargs = parts.len() - 1;

        for i in 1..parts.len() {
            operands.push(parts[i].to_string());
            operands[i - 1].retain(|c| !c.is_whitespace() && c != ',');
        }

        let instr = match nargs {
            1 => {
                let arg = match Self::parse_argument(&operands[0]) {
                    Some(x) => x,
                    _ => return None,
                };
                match opcode {
                    "jmp" => Instruction::Jmp(arg),
                    "cjmp" => Instruction::CJmp(arg),
                    "cnjmp" => Instruction::CnJmp(arg),
                    "answer" => Instruction::Answer(arg),
                    _ => return None,
                }
            }
            2 => {
                // For store instructions arguments are swapped
                let (reg, arg) = match opcode {
                    "store.b" | "store.w" => (
                        Self::parse_register(&operands[1]),
                        Self::parse_argument(&operands[0]),
                    ),
                    _ => (
                        Self::parse_register(&operands[0]),
                        Self::parse_argument(&operands[1]),
                    ),
                };

                if reg.is_none() || arg.is_none() {
                    return None;
                }
                let reg = reg.unwrap();
                let arg = arg.unwrap();

                match opcode {
                    "not" => Instruction::Not(reg, arg),
                    "cmpe" => Instruction::CmpE(reg, arg),
                    "cmpa" => Instruction::CmpA(reg, arg),
                    "cmpae" => Instruction::CmpAE(reg, arg),
                    "cmpg" => Instruction::CmpG(reg, arg),
                    "cmpge" => Instruction::CmpGE(reg, arg),
                    "mov" => Instruction::Mov(reg, arg),
                    "cmov" => Instruction::CMov(reg, arg),
                    "load.b" => Instruction::LoadB(reg, arg),
                    "load.w" => Instruction::LoadW(reg, arg),
                    "read" => Instruction::Read(reg, arg),
                    "store.b" => Instruction::StoreW(arg, reg),
                    "store.w" => Instruction::StoreW(arg, reg),
                    _ => return None,
                }
            }
            3 => {
                let reg1 = Self::parse_register(&operands[0]);
                let reg2 = Self::parse_register(&operands[1]);
                let arg = Self::parse_argument(&operands[2]);

                if reg1.is_none() || reg2.is_none() || arg.is_none() {
                    return None;
                }
                let reg1 = reg1.unwrap();
                let reg2 = reg2.unwrap();
                let arg = arg.unwrap();

                match opcode {
                    "and" => Instruction::And(reg1, reg2, arg),
                    "or" => Instruction::Or(reg1, reg2, arg),
                    "xor" => Instruction::Xor(reg1, reg2, arg),
                    "add" => Instruction::Add(reg1, reg2, arg),
                    "sub" => Instruction::Sub(reg1, reg2, arg),
                    "mull" => Instruction::MulL(reg1, reg2, arg),
                    "umulh" => Instruction::UMulH(reg1, reg2, arg),
                    "smulh" => Instruction::SMulH(reg1, reg2, arg),
                    "udiv" => Instruction::UDiv(reg1, reg2, arg),
                    "umod" => Instruction::UMod(reg1, reg2, arg),
                    "shl" => Instruction::Shl(reg1, reg2, arg),
                    "shr" => Instruction::Shr(reg1, reg2, arg),

                    _ => return None,
                }
            }
            _ => return None,
        };

        Some(instr)
    }

    /// Parse value
    fn parse_immediate(s: &str) -> Option<i64> {
        match s.parse::<i64>() {
            Ok(x) => Some(x),
            _ => None,
        }
    }

    /// Parse current argument
    fn parse_argument(s: &str) -> Option<Argument> {
        if let Some(reg) = Self::parse_register(s) {
            Some(Argument::Reg(reg))
        } else if let Some(label) = Self::parse_label_ident(s) {
            Some(Argument::Label(label))
        } else {
            Self::parse_immediate(s).map(Argument::Imm)
        }
    }

    /// Parse registers
    fn parse_register(s: &str) -> Option<Register> {
        if Self::starts_with(s, 'r') {
            s[1..].parse::<u16>().map(|index| Register { index }).ok()
        } else {
            None
        }
    }

    /// Parse whitespaces
    fn parse_whitespace(line: &str) -> Option<()> {
        if line.split_whitespace().collect::<String>().is_empty() {
            Some(())
        } else {
            None
        }
    }

    /// Parse labels
    fn parse_label(line: &str) -> Option<String> {
        if Self::ends_with(line, ':') {
            Self::parse_label_ident(&line[..line.len() - 1])
        } else {
            None
        }
    }

    /// Parse the label identifier
    fn parse_label_ident(s: &str) -> Option<String> {
        lazy_static! {
            static ref RE: Regex = Regex::new("_[0-9a-zA-Z_]+").unwrap();
        }

        if RE.is_match(s) {
            Some(s.to_string())
        } else {
            None
        }
    }

    /// Parse comments
    fn parse_comment(line: &str) -> Option<()> {
        if Self::starts_with(line, ';') {
            Some(())
        } else {
            None
        }
    }

    /// Check if line starts with designated character
    fn starts_with(line: &str, c: char) -> bool {
        match line.chars().next() {
            Some(x) => x == c,
            None => false,
        }
    }

    /// Check if line ends with designated character
    fn ends_with(line: &str, c: char) -> bool {
        let n = line.len() - 1;
        match line.chars().nth(n) {
            Some(x) => x == c,
            None => false,
        }
    }

    /// Read lines from the program files
    fn read_lines<P>(filename: P) -> Result<io::Lines<io::BufReader<File>>, io::Error>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }
}
