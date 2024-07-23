// File: crates/icn_vm/src/coop_vm.rs

use std::collections::HashMap;
use icn_common::{IcnResult, IcnError};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
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
    Lt,
    Lte,
    Gt,
    Gte,
    And,
    Or,
    Not,
    Store(String),
    Load(String),
    Call(String),
    Return,
    JumpIf(usize),
    Jump(usize),
    CreateList,
    AppendList,
    GetListItem,
    SetListItem,
    CreateMap,
    SetMapItem,
    GetMapItem,
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

    pub fn run(&mut self) -> IcnResult<()> {
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
            Opcode::Pop => { self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))? ; }
            Opcode::Add => self.binary_op(|a, b| a + b)?,
            Opcode::Sub => self.binary_op(|a, b| a - b)?,
            Opcode::Mul => self.binary_op(|a, b| a * b)?,
            Opcode::Div => self.binary_op(|a, b| a / b)?,
            Opcode::Mod => self.binary_op(|a, b| a % b)?,
            Opcode::Eq => self.compare_op(|a, b| a == b)?,
            Opcode::Neq => self.compare_op(|a, b| a != b)?,
            Opcode::Lt => self.compare_op(|a, b| a < b)?,
            Opcode::Lte => self.compare_op(|a, b| a <= b)?,
            Opcode::Gt => self.compare_op(|a, b| a > b)?,
            Opcode::Gte => self.compare_op(|a, b| a >= b)?,
            Opcode::And => self.logic_op(|a, b| a && b)?,
            Opcode::Or => self.logic_op(|a, b| a || b)?,
            Opcode::Not => {
                let a = self.pop_bool()?;
                self.stack.push(Value::Bool(!a));
            }
            Opcode::Store(name) => {
                let value = self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))?;
                self.memory.insert(name, value);
            }
            Opcode::Load(name) => {
                let value = self.memory.get(&name).ok_or(IcnError::VM("Variable not found".into()))?.clone();
                self.stack.push(value);
            }
            Opcode::Call(func_name) => {
                let func_pc = self.functions.get(&func_name).ok_or(IcnError::VM("Function not found".into()))?;
                self.call_stack.push(self.pc);
                self.pc = *func_pc;
            }
            Opcode::Return => {
                self.pc = self.call_stack.pop().ok_or(IcnError::VM("Return without call".into()))?;
            }
            Opcode::JumpIf(target) => {
                let condition = self.pop_bool()?;
                if condition {
                    self.pc = target;
                }
            }
            Opcode::Jump(target) => {
                self.pc = target;
            }
            Opcode::CreateList => self.stack.push(Value::List(Vec::new())),
            Opcode::AppendList => {
                let value = self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))?;
                if let Some(Value::List(list)) = self.stack.last_mut() {
                    list.push(value);
                } else {
                    return Err(IcnError::VM("Expected list on top of stack".into()));
                }
            }
            Opcode::GetListItem => {
                let index = self.pop_int()?;
                if let Some(Value::List(list)) = self.stack.pop() {
                    let item = list.get(index as usize).ok_or(IcnError::VM("List index out of bounds".into()))?.clone();
                    self.stack.push(item);
                } else {
                    return Err(IcnError::VM("Expected list on top of stack".into()));
                }
            }
            Opcode::SetListItem => {
                let value = self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))?;
                let index = self.pop_int()?;
                if let Some(Value::List(list)) = self.stack.last_mut() {
                    if (index as usize) < list.len() {
                        list[index as usize] = value;
                    } else {
                        return Err(IcnError::VM("List index out of bounds".into()));
                    }
                } else {
                    return Err(IcnError::VM("Expected list on top of stack".into()));
                }
            }
            Opcode::CreateMap => self.stack.push(Value::Map(HashMap::new())),
            Opcode::SetMapItem => {
                let value = self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))?;
                let key = self.pop_string()?;
                if let Some(Value::Map(map)) = self.stack.last_mut() {
                    map.insert(key, value);
                } else {
                    return Err(IcnError::VM("Expected map on top of stack".into()));
                }
            }
            Opcode::GetMapItem => {
                let key = self.pop_string()?;
                if let Some(Value::Map(map)) = self.stack.pop() {
                    let value = map.get(&key).ok_or(IcnError::VM("Key not found in map".into()))?.clone();
                    self.stack.push(value);
                } else {
                    return Err(IcnError::VM("Expected map on top of stack".into()));
                }
            }
            Opcode::Vote(proposal_id) => {
                let vote = self.pop_bool()?;
                // In a real implementation, this would interact with the governance system
                println!("Voting {} on proposal {}", if vote { "Yes" } else { "No" }, proposal_id);
            }
            Opcode::AllocateResource(resource_id) => {
                let amount = self.pop_int()?;
                // In a real implementation, this would interact with a resource management system
                println!("Allocating {} units of resource {}", amount, resource_id);
            }
            Opcode::UpdateReputation(address) => {
                let change = self.pop_int()?;
                // In a real implementation, this would interact with the reputation system
                println!("Updating reputation of {} by {}", address, change);
            }
            Opcode::CreateProposal => {
                let description = self.pop_string()?;
                // In a real implementation, this would interact with the governance system
                println!("Creating proposal: {}", description);
                self.stack.push(Value::String("new_proposal_id".to_string()));
            }
            Opcode::GetProposalStatus => {
                let proposal_id = self.pop_string()?;
                // In a real implementation, this would interact with the governance system
                println!("Getting status of proposal: {}", proposal_id);
                self.stack.push(Value::String("Active".to_string()));
            }
            Opcode::Emit(event_name) => {
                let event_data = self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))?;
                // In a real implementation, this would emit an event to the blockchain
                println!("Emitting event {}: {:?}", event_name, event_data);
            }
        }
        Ok(())
    }

    fn binary_op<F>(&mut self, op: F) -> IcnResult<()>
    where
        F: Fn(f64, f64) -> f64,
    {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        self.stack.push(Value::Float(op(a, b)));
        Ok(())
    }

    fn compare_op<F>(&mut self, op: F) -> IcnResult<()>
    where
        F: Fn(&Value, &Value) -> bool,
    {
        let b = self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))?;
        let a = self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))?;
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

    fn pop_number(&mut self) -> IcnResult<f64> {
        match self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))? {
            Value::Int(i) => Ok(i as f64),
            Value::Float(f) => Ok(f),
            _ => Err(IcnError::VM("Expected number".into())),
        }
    }

    fn pop_int(&mut self) -> IcnResult<i64> {
        match self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))? {
            Value::Int(i) => Ok(i),
            _ => Err(IcnError::VM("Expected integer".into())),
        }
    }

    fn pop_bool(&mut self) -> IcnResult<bool> {
        match self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))? {
            Value::Bool(b) => Ok(b),
            _ => Err(IcnError::VM("Expected boolean".into())),
        }
    }

    fn pop_string(&mut self) -> IcnResult<String> {
        match self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))? {
            Value::String(s) => Ok(s),
            _ => Err(IcnError::VM("Expected string".into())),
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

        assert_eq!(vm.stack, vec![Value::Float(16.0)]);
    }

    #[test]
    fn test_control_flow() {
        let program = vec![
            Opcode::Push(Value::Int(10)),
            Opcode::Push(Value::Int(5)),
            Opcode::Lt,
            Opcode::JumpIf(6),
            Opcode::Push(Value::String("Less".to_string())),
            Opcode::Jump(7),
            Opcode::Push(Value::String("Greater or Equal".to_string())),
        ];

        let mut vm = CoopVM::new(program);
        vm.run().unwrap();

        assert_eq!(vm.stack, vec![Value::String("Greater or Equal".to_string())]);
    }

    #[test]
    fn test_function_call() {
        let program = vec![
            Opcode::Push(Value::Int(5)),
            Opcode::Call("square".to_string()),
            Opcode::Push(Value::Int(3)),
            Opcode::Add,
            Opcode::Return,
            // square function
            Opcode::Mul,
            Opcode::Return,
        ];

        let mut vm = CoopVM::new(program);
        vm.register_function("square".to_string(), 5);
        vm.run().unwrap();

        assert_eq!(vm.stack, vec![Value::Float(28.0)]);
    }

    // File: crates/icn_vm/src/coop_vm.rs

    #[test]
    fn test_list_operations() {
        let program = vec![
            Opcode::CreateList,
            Opcode::Push(Value::Int(1)),
            Opcode::AppendList,
            Opcode::Push(Value::Int(2)),
            Opcode::AppendList,
            Opcode::Push(Value::Int(3)),
            Opcode::AppendList,
            Opcode::Push(Value::Int(1)),
            Opcode::GetListItem,
            Opcode::Push(Value::Int(10)),
            Opcode::Push(Value::Int(0)),
            Opcode::SetListItem,
        ];

        let mut vm = CoopVM::new(program);
        vm.run().unwrap();

        assert_eq!(vm.stack, vec![
            Value::List(vec![Value::Int(10), Value::Int(2), Value::Int(3)]),
            Value::Int(2)
        ]);
    }

    #[test]
    fn test_map_operations() {
        let program = vec![
            Opcode::CreateMap,
            Opcode::Push(Value::String("key1".to_string())),
            Opcode::Push(Value::Int(42)),
            Opcode::SetMapItem,
            Opcode::Push(Value::String("key2".to_string())),
            Opcode::Push(Value::String("value".to_string())),
            Opcode::SetMapItem,
            Opcode::Push(Value::String("key1".to_string())),
            Opcode::GetMapItem,
        ];

        let mut vm = CoopVM::new(program);
        vm.run().unwrap();

        let mut expected_map = HashMap::new();
        expected_map.insert("key1".to_string(), Value::Int(42));
        expected_map.insert("key2".to_string(), Value::String("value".to_string()));

        assert_eq!(vm.stack, vec![
            Value::Map(expected_map),
            Value::Int(42)
        ]);
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

        assert_eq!(vm.stack, vec![
            Value::String("new_proposal_id".to_string()),
            Value::String("Active".to_string())
        ]);
    }

    #[test]
    fn test_error_handling() {
        let program = vec![
            Opcode::Push(Value::Int(5)),
            Opcode::Push(Value::String("not a number".to_string())),
            Opcode::Add,
        ];

        let mut vm = CoopVM::new(program);
        let result = vm.run();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "VM error: Expected number");
    }
}