// File: src/vm.rs

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Address(String),
    List(Vec<Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Address(a) => write!(f, "Address({})", a),
            Value::List(l) => write!(f, "{:?}", l),
        }
    }
}

#[derive(Debug, Clone)]
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
    CreateList,
    AppendList,
    GetListItem,
    SetListItem,
    Vote(String),
    AllocateResource(String),
    UpdateReputation(String),
    CreateProposal,
    GetProposalStatus,
    Emit(String),
}

pub struct CoopVM {
    stack: Vec<Value>,
    memory: HashMap<String, Value>,
    program: Vec<Opcode>,
    pc: usize,
    call_stack: Vec<usize>,
    functions: HashMap<String, usize>,
}

impl CoopVM {
    pub fn new(program: Vec<Opcode>) -> Self {
        CoopVM {
            stack: Vec::new(),
            memory: HashMap::new(),
            program,
            pc: 0,
            call_stack: Vec::new(),
            functions: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.pc < self.program.len() {
            self.execute_instruction()?;
            self.pc += 1;
        }
        Ok(())
    }

    fn execute_instruction(&mut self) -> Result<(), String> {
        let current_instruction = &self.program[self.pc].clone();
        match current_instruction {
            Opcode::Push(value) => self.stack.push(value.clone()),
            Opcode::Pop => {
                self.stack.pop().ok_or("Stack underflow")?;
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
            Opcode::And => self.logic_op(|a, b| a && b)?,
            Opcode::Or => self.logic_op(|a, b| a || b)?,
            Opcode::Not => {
                let a = self.pop_bool()?;
                self.stack.push(Value::Bool(!a));
            }
            Opcode::Store(name) => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                self.memory.insert(name.clone(), value);
            }
            Opcode::Load(name) => {
                let value = self.memory.get(name).ok_or("Variable not found")?.clone();
                self.stack.push(value);
            }
            Opcode::JumpIf(target) => {
                if self.pop_bool()? {
                    self.pc = *target - 1;
                }
            }
            Opcode::Jump(target) => {
                self.pc = *target - 1;
            }
            Opcode::Call(func_name) => {
                let func_pc = self.functions.get(func_name).ok_or("Function not found")?;
                self.call_stack.push(self.pc);
                self.pc = *func_pc - 1;
            }
            Opcode::Return => {
                self.pc = self.call_stack.pop().ok_or("Return without call")?;
            }
            Opcode::CreateList => {
                self.stack.push(Value::List(Vec::new()));
            }
            Opcode::AppendList => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                if let Some(Value::List(list)) = self.stack.last_mut() {
                    list.push(value);
                } else {
                    return Err("Expected list on top of stack".to_string());
                }
            }
            Opcode::GetListItem => {
                let index = self.pop_int()?;
                if let Some(Value::List(list)) = self.stack.pop() {
                    let item = list.get(index as usize).ok_or("List index out of bounds")?.clone();
                    self.stack.push(item);
                } else {
                    return Err("Expected list on top of stack".to_string());
                }
            }
            Opcode::SetListItem => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                let index = self.pop_int()?;
                if let Some(Value::List(list)) = self.stack.last_mut() {
                    if (index as usize) < list.len() {
                        list[index as usize] = value;
                    } else {
                        return Err("List index out of bounds".to_string());
                    }
                } else {
                    return Err("Expected list on top of stack".to_string());
                }
            }
            Opcode::Vote(proposal_id) => {
                let vote = self.pop_bool()?;
                println!("Voting {} on proposal {}", if vote { "Yes" } else { "No" }, proposal_id);
            }
            Opcode::AllocateResource(resource_id) => {
                let amount = self.pop_int()?;
                println!("Allocating {} units of resource {}", amount, resource_id);
            }
            Opcode::UpdateReputation(address) => {
                let change = self.pop_int()?;
                println!("Updating reputation of {} by {}", address, change);
            }
            Opcode::CreateProposal => {
                let description = self.pop_string()?;
                println!("Creating proposal: {}", description);
                self.stack.push(Value::String("new_proposal_id".to_string()));
            }
            Opcode::GetProposalStatus => {
                let proposal_id = self.pop_string()?;
                println!("Getting status of proposal: {}", proposal_id);
                self.stack.push(Value::String("Active".to_string()));
            }
            Opcode::Emit(event_name) => {
                let event_data = self.stack.pop().ok_or("Stack underflow")?;
                println!("Emitting event {}: {}", event_name, event_data);
            }
        }
        Ok(())
    }

    fn binary_op<F>(&mut self, op: F) -> Result<(), String>
    where
        F: Fn(i64, i64) -> i64,
    {
        let b = self.pop_int()?;
        let a = self.pop_int()?;
        self.stack.push(Value::Int(op(a, b)));
        Ok(())
    }

    fn compare_op<F>(&mut self, op: F) -> Result<(), String>
    where
        F: Fn(&Value, &Value) -> bool,
    {
        let b = self.stack.pop().ok_or("Stack underflow")?;
        let a = self.stack.pop().ok_or("Stack underflow")?;
        self.stack.push(Value::Bool(op(&a, &b)));
        Ok(())
    }

    fn logic_op<F>(&mut self, op: F) -> Result<(), String>
    where
        F: Fn(bool, bool) -> bool,
    {
        let b = self.pop_bool()?;
        let a = self.pop_bool()?;
        self.stack.push(Value::Bool(op(a, b)));
        Ok(())
    }

    fn pop_int(&mut self) -> Result<i64, String> {
        match self.stack.pop().ok_or("Stack underflow")? {
            Value::Int(i) => Ok(i),
            _ => Err("Expected integer value".to_string()),
        }
    }

    fn pop_bool(&mut self) -> Result<bool, String> {
        match self.stack.pop().ok_or("Stack underflow")? {
            Value::Bool(b) => Ok(b),
            _ => Err("Expected boolean value".to_string()),
        }
    }

    fn pop_string(&mut self) -> Result<String, String> {
        match self.stack.pop().ok_or("Stack underflow")? {
            Value::String(s) => Ok(s),
            _ => Err("Expected string value".to_string()),
        }
    }

    pub fn register_function(&mut self, name: String, pc: usize) {
        self.functions.insert(name, pc);
    }

    pub fn get_stack(&self) -> &Vec<Value> {
        &self.stack
    }

    pub fn get_memory(&self) -> &HashMap<String, Value> {
        &self.memory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
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
    fn test_store_and_load() {
        let program = vec![
            Opcode::Push(Value::Int(42)),
            Opcode::Store("x".to_string()),
            Opcode::Push(Value::Int(10)),
            Opcode::Load("x".to_string()),
            Opcode::Add,
        ];

        let mut vm = CoopVM::new(program);
        vm.run().unwrap();

        assert_eq!(vm.stack, vec![Value::Int(52)]);
    }

    #[test]
    fn test_conditional_jump() {
        let program = vec![
            Opcode::Push(Value::Bool(true)),
            Opcode::JumpIf(3),
            Opcode::Push(Value::Int(1)),
            Opcode::Push(Value::Int(2)),
            Opcode::Add,
        ];

        let mut vm = CoopVM::new(program);
        vm.run().unwrap();

        assert_eq!(vm.stack, vec![Value::Int(2)]);
    }

    #[test]
    fn test_list_operations() {
        let program = vec![
            Opcode::CreateList,
            Opcode::Push(Value::Int(1)),
            Opcode::AppendList,
            Opcode::Push(Value::Int(2)),
            Opcode::AppendList,
            Opcode::Push(Value::Int(0)),
            Opcode::GetListItem,
        ];

        let mut vm = CoopVM::new(program);
        vm.run().unwrap();

        assert_eq!(vm.stack, vec![Value::List(vec![Value::Int(1), Value::Int(2)]), Value::Int(1)]);
    }

    #[test]
    fn test_cooperative_operations() {
        let program = vec![
            Opcode::Push(Value::String("Proposal 1".to_string())),
            Opcode::CreateProposal,
            Opcode::Push(Value::Bool(true)),
            Opcode::Vote("proposal_1".to_string()),
            Opcode::Push(Value::Int(100)),
            Opcode::AllocateResource("computing_power".to_string()),
            Opcode::Push(Value::Int(5)),
            Opcode::UpdateReputation("user1".to_string()),
            Opcode::Push(Value::String("proposal_1".to_string())),
            Opcode::GetProposalStatus,
        ];

        let mut vm = CoopVM::new(program);
        vm.run().unwrap();

        assert_eq!(vm.stack, vec![Value::String("new_proposal_id".to_string()), Value::String("Active".to_string())]);
    }
}
