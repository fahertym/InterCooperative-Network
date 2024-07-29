use icn_common::{IcnError, IcnResult};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
            (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Opcode {
    Push(Value),
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
    And,
    Or,
    Not,
    Store(String),
    Load(String),
    JumpIf(usize),
    Jump(usize),
    Call(String),
    Return,
    NetNodeConnect,
    ChainBlockCreate,
    EconCurrencyMint,
    GovProposalSubmit,
    CoopMemberAdd,
    CommEventOrganize,
    VoteOnProposal,
    AllocateResource,
    UpdateReputation,
    CreateProposal,
    GetProposalStatus,
    EmitEvent,
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

    pub fn execute(&mut self) -> IcnResult<()> {
        while self.pc < self.program.len() {
            self.execute_instruction()?;
            self.pc += 1;
        }
        Ok(())
    }

    fn execute_instruction(&mut self) -> IcnResult<()> {
        let instruction = self.program[self.pc].clone();
        match instruction {
            Opcode::Push(value) => self.stack.push(value),
            Opcode::Pop => {
                self.stack.pop().ok_or_else(|| IcnError::Vm("Stack underflow".into()))?;
            }
            Opcode::Add => self.binary_op(|a, b| a + b)?,
            Opcode::Sub => self.binary_op(|a, b| a - b)?,
            Opcode::Mul => self.binary_op(|a, b| a * b)?,
            Opcode::Div => self.binary_op(|a, b| a / b)?,
            Opcode::Mod => self.binary_op(|a, b| a % b)?,
            Opcode::Eq => self.compare_op(|a, b| a == b)?,
            Opcode::Neq => self.compare_op(|a, b| a != b)?,
            Opcode::Gt => self.compare_op(|a, b| a > b)?,
            Opcode::Lt => self.compare_op(|a, b| a < b)?,
            Opcode::Gte => self.compare_op(|a, b| a >= b)?,
            Opcode::Lte => self.compare_op(|a, b| a <= b)?,
            Opcode::And => {
                let b = self.pop_bool()?;
                let a = self.pop_bool()?;
                self.stack.push(Value::Bool(a && b));
            }
            Opcode::Or => {
                let b = self.pop_bool()?;
                let a = self.pop_bool()?;
                self.stack.push(Value::Bool(a || b));
            }
            Opcode::Not => {
                let a = self.pop_bool()?;
                self.stack.push(Value::Bool(!a));
            }
            Opcode::Store(name) => {
                let value = self.stack.pop().ok_or_else(|| IcnError::Vm("Stack underflow".into()))?;
                self.memory.insert(name, value);
            }
            Opcode::Load(name) => {
                let value = self.memory.get(&name).ok_or_else(|| IcnError::Vm("Variable not found".into()))?.clone();
                self.stack.push(value);
            }
            Opcode::JumpIf(target) => {
                let condition = self.pop_bool()?;
                if condition {
                    self.pc = target - 1; // -1 because pc will be incremented after this
                }
            }
            Opcode::Jump(target) => {
                self.pc = target - 1; // -1 because pc will be incremented after this
            }
            Opcode::Call(_) => return Err(IcnError::Vm("Function calls not implemented".into())),
            Opcode::Return => return Ok(()),
            Opcode::NetNodeConnect => println!("Executing NetNodeConnect"),
            Opcode::ChainBlockCreate => println!("Executing ChainBlockCreate"),
            Opcode::EconCurrencyMint => println!("Executing EconCurrencyMint"),
            Opcode::GovProposalSubmit => println!("Executing GovProposalSubmit"),
            Opcode::CoopMemberAdd => println!("Executing CoopMemberAdd"),
            Opcode::CommEventOrganize => println!("Executing CommEventOrganize"),
            Opcode::VoteOnProposal => println!("Executing VoteOnProposal"),
            Opcode::AllocateResource => println!("Executing AllocateResource"),
            Opcode::UpdateReputation => println!("Executing UpdateReputation"),
            Opcode::CreateProposal => println!("Executing CreateProposal"),
            Opcode::GetProposalStatus => println!("Executing GetProposalStatus"),
            Opcode::EmitEvent => println!("Executing EmitEvent"),
        }
        Ok(())
    }

    fn binary_op<F>(&mut self, op: F) -> IcnResult<()>
    where
        F: Fn(f64, f64) -> f64,
    {
        let b = self.pop_float()?;
        let a = self.pop_float()?;
        self.stack.push(Value::Float(op(a, b)));
        Ok(())
    }

    fn compare_op<F>(&mut self, op: F) -> IcnResult<()>
    where
        F: Fn(&Value, &Value) -> bool,
    {
        let b = self.stack.pop().ok_or_else(|| IcnError::Vm("Stack underflow".into()))?;
        let a = self.stack.pop().ok_or_else(|| IcnError::Vm("Stack underflow".into()))?;
        self.stack.push(Value::Bool(op(&a, &b)));
        Ok(())
    }

    fn pop_float(&mut self) -> IcnResult<f64> {
        match self.stack.pop().ok_or_else(|| IcnError::Vm("Stack underflow".into()))? {
            Value::Float(f) => Ok(f),
            Value::Int(i) => Ok(i as f64),
            _ => Err(IcnError::Vm("Expected float value".into())),
        }
    }

    fn pop_bool(&mut self) -> IcnResult<bool> {
        match self.stack.pop().ok_or_else(|| IcnError::Vm("Stack underflow".into()))? {
            Value::Bool(b) => Ok(b),
            _ => Err(IcnError::Vm("Expected boolean value".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let program = vec![
            Opcode::Push(Value::Float(5.0)),
            Opcode::Push(Value::Float(3.0)),
            Opcode::Add,
            Opcode::Push(Value::Float(2.0)),
            Opcode::Mul,
        ];

        let mut vm = CoopVM::new(program);
        assert!(vm.execute().is_ok());
        
        assert_eq!(vm.stack, vec![Value::Float(16.0)]);
    }

    #[test]
    fn test_comparison_operations() {
        let program = vec![
            Opcode::Push(Value::Int(5)),
            Opcode::Push(Value::Int(3)),
            Opcode::Gt,
            Opcode::Push(Value::Int(5)),
            Opcode::Push(Value::Int(5)),
            Opcode::Eq,
            Opcode::And,
        ];

        let mut vm = CoopVM::new(program);
        assert!(vm.execute().is_ok());
        
        assert_eq!(vm.stack, vec![Value::Bool(true)]);
    }

    #[test]
    fn test_logical_operations() {
        let program = vec![
            Opcode::Push(Value::Bool(true)),
            Opcode::Push(Value::Bool(false)),
            Opcode::Or,
            Opcode::Push(Value::Bool(true)),
            Opcode::And,
            Opcode::Not,
        ];

        let mut vm = CoopVM::new(program);
        assert!(vm.execute().is_ok());
        
        assert_eq!(vm.stack, vec![Value::Bool(false)]);
    }

    #[test]
    fn test_conditional_jump() {
        let program = vec![
            Opcode::Push(Value::Bool(true)),
            Opcode::JumpIf(4),
            Opcode::Push(Value::Int(1)),
            Opcode::Jump(5),
            Opcode::Push(Value::Int(2)),
        ];

        let mut vm = CoopVM::new(program);
        assert!(vm.execute().is_ok());
        
        assert_eq!(vm.stack, vec![Value::Int(2)]);
    }

    #[test]
    fn test_store_and_load() {
        let program = vec![
            Opcode::Push(Value::Int(42)),
            Opcode::Store("x".to_string()),
            Opcode::Push(Value::Int(10)),
            Opcode::Load("x".to_string()),
            Opcode::Add,
        ];

        let mut vm = CoopVM::new(program);
        assert!(vm.execute().is_ok());
        
        assert_eq!(vm.stack, vec![Value::Float(52.0)]);
    }

    #[test]
    fn test_error_handling() {
        let program = vec![
            Opcode::Push(Value::Int(1)),
            Opcode::Pop,
            Opcode::Pop, // This should cause an error
        ];

        let mut vm = CoopVM::new(program);
        assert!(vm.execute().is_err());
    }

    #[test]
    fn test_complex_program() {
        let program = vec![
            Opcode::Push(Value::Int(10)),
            Opcode::Store("x".to_string()),
            Opcode::Push(Value::Int(0)),
            Opcode::Store("sum".to_string()),
            Opcode::Push(Value::Int(1)),
            Opcode::Store("i".to_string()),
            // Loop start
            Opcode::Load("i".to_string()),
            Opcode::Load("x".to_string()),
            Opcode::Lte,
            Opcode::JumpIf(17), // Jump to end if false
            Opcode::Load("sum".to_string()),
            Opcode::Load("i".to_string()),
            Opcode::Add,
            Opcode::Store("sum".to_string()),
            Opcode::Load("i".to_string()),
            Opcode::Push(Value::Int(1)),
            Opcode::Add,
            Opcode::Store("i".to_string()),
            Opcode::Jump(6), // Jump back to loop start
            // Loop end
            Opcode::Load("sum".to_string()),
        ];

        let mut vm = CoopVM::new(program);
        assert!(vm.execute().is_ok());
        
        // Sum of numbers from 1 to 10 is 55
        assert_eq!(vm.stack, vec![Value::Float(55.0)]);
    }
}
