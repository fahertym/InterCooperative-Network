use super::opcode::{Opcode, Value};
use std::collections::HashMap;

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

    pub fn load_program(&mut self, program: Vec<Opcode>) {
        self.program = program;
        self.pc = 0;
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.pc < self.program.len() {
            self.execute_instruction()?;
            self.pc += 1;
        }
        Ok(())
    }

    fn execute_instruction(&mut self) -> Result<(), String> {
        let opcode = self.program[self.pc].clone();
        match opcode {
            Opcode::Push(value) => self.stack.push(value),
            Opcode::Pop => {
                self.stack.pop().ok_or("Stack underflow")?;
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
            Opcode::Return => return Ok(()), // For now, just return
            Opcode::Store(name) => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                self.memory.insert(name, value);
            }
            Opcode::Load(name) => {
                let value = self.memory.get(&name).ok_or("Variable not found")?.clone();
                self.stack.push(value);
            }
            Opcode::Call(_) => return Err("Function calls not implemented yet".to_string()),
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
                println!("Emitting event {}: {:?}", event_name, event_data);
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

    pub fn get_stack(&self) -> &Vec<Value> {
        &self.stack
    }

    pub fn get_memory(&self) -> &HashMap<String, Value> {
        &self.memory
    }
}
