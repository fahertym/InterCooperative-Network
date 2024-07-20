// File: icn_vm/src/lib.rs

use icn_types::{IcnResult, IcnError};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone)]
pub enum Opcode {
    Push(Value),
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
    And,
    Or,
    Not,
    Store(String),
    Load(String),
    Call(String),
    Return,
    JumpIf(usize),
    Jump(usize),
}

pub struct CoopVM {
    stack: Vec<Value>,
    memory: HashMap<String, Value>,
    program: Vec<Opcode>,
    pc: usize,
}

impl CoopVM {
    pub fn new(program: Vec<Opcode>) -> Self {
        CoopVM {
            stack: Vec::new(),
            memory: HashMap::new(),
            program,
            pc: 0,
        }
    }

    pub fn run(&mut self) -> IcnResult<()> {
        while self.pc < self.program.len() {
            self.execute_instruction()?;
            self.pc += 1;
        }
        Ok(())
    }

    fn execute_instruction(&mut self) -> IcnResult<()> {
        let opcode = self.program[self.pc].clone();
        match opcode {
            Opcode::Push(value) => self.stack.push(value),
            Opcode::Pop => {
                self.stack.pop().ok_or(IcnError::VM("Stack underflow".to_string()))?;
            }
            Opcode::Add => self.binary_op(|a, b| a + b)?,
            Opcode::Sub => self.binary_op(|a, b| a - b)?,
            Opcode::Mul => self.binary_op(|a, b| a * b)?,
            Opcode::Div => self.binary_op(|a, b| a / b)?,
            Opcode::Eq => self.compare_op(|a, b| a == b)?,
            Opcode::Lt => self.compare_op(|a, b| a < b)?,
            Opcode::Gt => self.compare_op(|a, b| a > b)?,
            Opcode::And => self.logic_op(|a, b| a && b)?,
            Opcode::Or => self.logic_op(|a, b| a || b)?,
            Opcode::Not => {
                let a = self.pop_bool()?;
                self.stack.push(Value::Bool(!a));
            }
            Opcode::Store(name) => {
                let value = self.stack.pop().ok_or(IcnError::VM("Stack underflow".to_string()))?;
                self.memory.insert(name, value);
            }
            Opcode::Load(name) => {
                let value = self.memory.get(&name).ok_or(IcnError::VM("Variable not found".to_string()))?.clone();
                self.stack.push(value);
            }
            Opcode::Call(_) => return Err(IcnError::VM("Function calls not implemented yet".to_string())),
            Opcode::Return => return Ok(()),
            Opcode::JumpIf(target) => {
                let condition = self.pop_bool()?;
                if condition {
                    self.pc = target - 1; // -1 because pc will be incremented after this
                }
            }
            Opcode::Jump(target) => {
                self.pc = target - 1; // -1 because pc will be incremented after this
            }
        }
        Ok(())
    }

    fn binary_op<F>(&mut self, op: F) -> IcnResult<()>
    where
        F: Fn(i64, i64) -> i64,
    {
        let b = self.pop_int()?;
        let a = self.pop_int()?;
        self.stack.push(Value::Int(op(a, b)));
        Ok(())
    }

    fn compare_op<F>(&mut self, op: F) -> IcnResult<()>
    where
        F: Fn(&Value, &Value) -> bool,
    {
        let b = self.stack.pop().ok_or(IcnError::VM("Stack underflow".to_string()))?;
        let a = self.stack.pop().ok_or(IcnError::VM("Stack underflow".to_string()))?;
        self.stack.push(Value::Bool(op(&a, &b)));
        Ok(())
    }

    fn logic_op<F>(&mut self, op: F) -> IcnResult<()>
    where
        F: Fn(bool, bool) -> bool,
    {
        let b = self.pop_bool()?;
        let a = self.pop_bool()?;
        self.stack.push(Value::Bool(op(a, b)));
        Ok(())
    }

    fn pop_int(&mut self) -> IcnResult<i64> {
        match self.stack.pop().ok_or(IcnError::VM("Stack underflow".to_string()))? {
            Value::Int(i) => Ok(i),
            _ => Err(IcnError::VM("Expected integer value".to_string())),
        }
    }

    fn pop_bool(&mut self) -> IcnResult<bool> {
        match self.stack.pop().ok_or(IcnError::VM("Stack underflow".to_string()))? {
            Value::Bool(b) => Ok(b),
            _ => Err(IcnError::VM("Expected boolean value".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_program() {
        let program = vec![
            Opcode::Push(Value::Int(5)),
            Opcode::Push(Value::Int(3)),
            Opcode::Add,
            Opcode::Push(Value::Int(2)),
            Opcode::Mul,
        ];

        let mut vm = CoopVM::new(program);
        vm.run().unwrap();

        assert_eq!(vm.stack, vec![Value::Int(16)]);
    }

    #[test]
    fn test_conditional_jump() {
        let program = vec![
            Opcode::Push(Value::Bool(true)),
            Opcode::JumpIf(4),
            Opcode::Push(Value::Int(1)),
            Opcode::Jump(5),
            Opcode::Push(Value::Int(2)),
            Opcode::Push(Value::Int(3)),
            Opcode::Add,
        ];

        let mut vm = CoopVM::new(program);
        vm.run().unwrap();

        assert_eq!(vm.stack, vec![Value::Int(5)]);
    }
}