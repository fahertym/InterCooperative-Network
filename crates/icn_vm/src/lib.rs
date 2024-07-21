use icn_common::{IcnResult, IcnError};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[derive(Clone, Debug)]
pub enum Opcode {
    Push(Value),
    Pop,
    Add,
    Sub,
    Mul,
    Div,
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
    JumpIf(usize),
    Jump(usize),
}

pub struct CoopVM {
    stack: Vec<Value>,
    memory: HashMap<String, Value>,
    program: Vec<Opcode>,
    pc: usize,
    context: Option<ContractContext>,
}

pub struct ContractContext {
    pub balances: HashMap<String, f64>,
    pub votes: HashMap<String, bool>,
    pub reputation: HashMap<String, i32>,
}

impl CoopVM {
    pub fn new(program: Vec<Opcode>) -> Self {
        CoopVM {
            stack: Vec::new(),
            memory: HashMap::new(),
            program,
            pc: 0,
            context: None,
        }
    }

    pub fn load_program(&mut self, program: Vec<Opcode>) {
        self.program = program;
        self.pc = 0;
    }

    pub fn set_context(&mut self, context: &ContractContext) {
        self.context = Some(ContractContext {
            balances: context.balances.clone(),
            votes: context.votes.clone(),
            reputation: context.reputation.clone(),
        });
    }

    pub fn run(&mut self) -> IcnResult<()> {
        while self.pc < self.program.len() {
            self.execute_instruction()?;
            self.pc += 1;
        }
        Ok(())
    }

    fn execute_instruction(&mut self) -> IcnResult<()> {
        let instruction = &self.program[self.pc];
        match instruction {
            Opcode::Push(value) => self.stack.push(value.clone()),
            Opcode::Pop => { self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))? ; }
            Opcode::Add => self.binary_op(|a, b| a + b)?,
            Opcode::Sub => self.binary_op(|a, b| a - b)?,
            Opcode::Mul => self.binary_op(|a, b| a * b)?,
            Opcode::Div => self.binary_op(|a, b| a / b)?,
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
                self.memory.insert(name.clone(), value);
            }
            Opcode::Load(name) => {
                let value = self.memory.get(name).ok_or(IcnError::VM("Variable not found".into()))?.clone();
                self.stack.push(value);
            }
            Opcode::Call(function) => {
                let result = self.execute_built_in(function)?;
                if let Some(value) = result {
                    self.stack.push(value);
                }
            }
            Opcode::JumpIf(target) => {
                if self.pop_bool()? {
                    self.pc = *target - 1; // -1 because pc will be incremented after this instruction
                }
            }
            Opcode::Jump(target) => {
                self.pc = *target - 1; // -1 because pc will be incremented after this instruction
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
        F: Fn(f64, f64) -> bool,
    {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        self.stack.push(Value::Bool(op(a, b)));
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

    fn execute_built_in(&mut self, function: &str) -> IcnResult<Option<Value>> {
        match function {
            "transfer" => self.transfer(),
            "vote" => self.vote(),
            "get_balance" => self.get_balance(),
            "update_reputation" => self.update_reputation(),
            _ => Err(IcnError::VM(format!("Unknown built-in function: {}", function))),
        }
    }

    fn transfer(&mut self) -> IcnResult<Option<Value>> {
        let amount = self.pop_number()?;
        let to = self.pop_string()?;
        let from = self.pop_string()?;

        let context = self.context.as_mut().ok_or(IcnError::VM("No context set".into()))?;
        
        let from_balance = context.balances.entry(from.clone()).or_insert(0.0);
        if *from_balance < amount {
            return Err(IcnError::VM("Insufficient balance".into()));
        }
        *from_balance -= amount;

        let to_balance = context.balances.entry(to.clone()).or_insert(0.0);
        *to_balance += amount;

        Ok(None)
    }

    fn vote(&mut self) -> IcnResult<Option<Value>> {
        let vote = self.pop_bool()?;
        let proposal = self.pop_string()?;

        let context = self.context.as_mut().ok_or(IcnError::VM("No context set".into()))?;
        context.votes.insert(proposal, vote);

        Ok(None)
    }

    fn get_balance(&mut self) -> IcnResult<Option<Value>> {
        let address = self.pop_string()?;

        let context = self.context.as_ref().ok_or(IcnError::VM("No context set".into()))?;
        let balance = *context.balances.get(&address).unwrap_or(&0.0);

        Ok(Some(Value::Float(balance)))
    }

    fn update_reputation(&mut self) -> IcnResult<Option<Value>> {
        let change = self.pop_number()? as i32;
        let address = self.pop_string()?;

        let context = self.context.as_mut().ok_or(IcnError::VM("No context set".into()))?;
        let reputation = context.reputation.entry(address).or_insert(0);
        *reputation += change;

        Ok(None)
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
    fn test_transfer() {
        let program = vec![
            Opcode::Push(Value::String("Alice".to_string())),
            Opcode::Push(Value::String("Bob".to_string())),
            Opcode::Push(Value::Float(50.0)),
            Opcode::Call("transfer".to_string()),
            Opcode::Push(Value::String("Alice".to_string())),
            Opcode::Call("get_balance".to_string()),
            Opcode::Push(Value::String("Bob".to_string())),
            Opcode::Call("get_balance".to_string()),
        ];

        let mut vm = CoopVM::new(program);
        let mut context = ContractContext {
            balances: HashMap::new(),
            votes: HashMap::new(),
            reputation: HashMap::new(),
        };
        context.balances.insert("Alice".to_string(), 100.0);
        context.balances.insert("Bob".to_string(), 0.0);
        vm.set_context(&context);

        vm.run().unwrap();

        assert_eq!(vm.stack, vec![Value::Float(50.0), Value::Float(50.0)]);
    }

    #[test]
    fn test_voting() {
        let program = vec![
            Opcode::Push(Value::String("proposal1".to_string())),
            Opcode::Push(Value::Bool(true)),
            Opcode::Call("vote".to_string()),
            Opcode::Push(Value::String("proposal2".to_string())),
            Opcode::Push(Value::Bool(false)),
            Opcode::Call("vote".to_string()),
        ];

        let mut vm = CoopVM::new(program);
        let context = ContractContext {
            balances: HashMap::new(),
            votes: HashMap::new(),
            reputation: HashMap::new(),
        };
        vm.set_context(&context);

        vm.run().unwrap();

        assert_eq!(vm.context.unwrap().votes.len(), 2);
        assert_eq!(vm.context.unwrap().votes.get("proposal1"), Some(&true));
        assert_eq!(vm.context.unwrap().votes.get("proposal2"), Some(&false));
    }
}