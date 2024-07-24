// crates/icn_vm/src/lib.rs

use icn_common::{IcnError, IcnResult};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Address(String),
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
    Jump(usize),
    JumpIfFalse(usize),
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

    pub fn run(&mut self) -> IcnResult<()> {
        while self.pc < self.program.len() {
            self.execute_instruction()?;
        }
        Ok(())
    }

    fn execute_instruction(&mut self) -> IcnResult<()> {
        let instruction = self.program[self.pc].clone();
        self.pc += 1;

        match instruction {
            Opcode::Push(value) => self.stack.push(value),
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
                self.memory.insert(name, value);
            }
            Opcode::Load(name) => {
                let value = self.memory.get(&name).ok_or(IcnError::VM("Variable not found".into()))?.clone();
                self.stack.push(value);
            }
            Opcode::Jump(target) => {
                self.pc = target;
            }
            Opcode::JumpIfFalse(target) => {
                let condition = self.pop_bool()?;
                if !condition {
                    self.pc = target;
                }
            }
            Opcode::Call(function) => {
                self.execute_function(&function)?;
            }
            Opcode::Return => {
                // For simplicity, we'll just stop execution
                self.pc = self.program.len();
            }
            Opcode::NetNodeConnect => self.net_node_connect()?,
            Opcode::ChainBlockCreate => self.chain_block_create()?,
            Opcode::EconCurrencyMint => self.econ_currency_mint()?,
            Opcode::GovProposalSubmit => self.gov_proposal_submit()?,
            Opcode::CoopMemberAdd => self.coop_member_add()?,
            Opcode::CommEventOrganize => self.comm_event_organize()?,
            Opcode::VoteOnProposal => self.vote_on_proposal()?,
            Opcode::AllocateResource => self.allocate_resource()?,
            Opcode::UpdateReputation => self.update_reputation()?,
            Opcode::CreateProposal => self.create_proposal()?,
            Opcode::GetProposalStatus => self.get_proposal_status()?,
            Opcode::EmitEvent => self.emit_event()?,
        }
        Ok(())
    }

    // Implement the binary_op, compare_op, logic_op, and pop_bool methods here...

    fn execute_function(&mut self, function: &str) -> IcnResult<()> {
        // Implement function execution logic here
        Ok(())
    }

    fn net_node_connect(&mut self) -> IcnResult<()> {
        let node2 = self.pop_string()?;
        let node1 = self.pop_string()?;
        println!("Connecting nodes: {} and {}", node1, node2);
        Ok(())
    }

    fn chain_block_create(&mut self) -> IcnResult<()> {
        let transactions = self.pop_list()?;
        println!("Creating block with {} transactions", transactions.len());
        Ok(())
    }

    fn econ_currency_mint(&mut self) -> IcnResult<()> {
        let currency_type = self.pop_string()?;
        let amount = self.pop_float()?;
        println!("Minting {} of currency type {}", amount, currency_type);
        Ok(())
    }

    fn gov_proposal_submit(&mut self) -> IcnResult<()> {
        let description = self.pop_string()?;
        println!("Submitting proposal: {}", description);
        Ok(())
    }

    fn coop_member_add(&mut self) -> IcnResult<()> {
        let member_id = self.pop_string()?;
        let coop_id = self.pop_string()?;
        println!("Adding member {} to cooperative {}", member_id, coop_id);
        Ok(())
    }

    fn comm_event_organize(&mut self) -> IcnResult<()> {
        let event_details = self.pop_string()?;
        println!("Organizing community event: {}", event_details);
        Ok(())
    }

    fn vote_on_proposal(&mut self) -> IcnResult<()> {
        let vote = self.pop_bool()?;
        let proposal_id = self.pop_string()?;
        println!("Voting {} on proposal {}", if vote { "Yes" } else { "No" }, proposal_id);
        Ok(())
    }

    fn allocate_resource(&mut self) -> IcnResult<()> {
        let amount = self.pop_int()?;
        let resource = self.pop_string()?;
        println!("Allocating {} units of resource {}", amount, resource);
        Ok(())
    }

    fn update_reputation(&mut self) -> IcnResult<()> {
        let change = self.pop_int()?;
        let address = self.pop_string()?;
        println!("Updating reputation of {} by {}", address, change);
        Ok(())
    }

    fn create_proposal(&mut self) -> IcnResult<()> {
        let description = self.pop_string()?;
        let title = self.pop_string()?;
        println!("Creating proposal: {} - {}", title, description);
        self.stack.push(Value::String("new_proposal_id".to_string()));
        Ok(())
    }

    fn get_proposal_status(&mut self) -> IcnResult<()> {
        let proposal_id = self.pop_string()?;
        println!("Getting status of proposal: {}", proposal_id);
        self.stack.push(Value::String("Active".to_string()));
        Ok(())
    }

    fn emit_event(&mut self) -> IcnResult<()> {
        let event_data = self.pop_string()?;
        let event_name = self.pop_string()?;
        println!("Emitting event {}: {}", event_name, event_data);
        Ok(())
    }

    fn pop_string(&mut self) -> IcnResult<String> {
        match self.stack.pop() {
            Some(Value::String(s)) => Ok(s),
            _ => Err(IcnError::VM("Expected string value".into())),
        }
    }

    fn pop_int(&mut self) -> IcnResult<i64> {
        match self.stack.pop() {
            Some(Value::Int(i)) => Ok(i),
            _ => Err(IcnError::VM("Expected integer value".into())),
        }
    }

    fn pop_float(&mut self) -> IcnResult<f64> {
        match self.stack.pop() {
            Some(Value::Float(f)) => Ok(f),
            _ => Err(IcnError::VM("Expected float value".into())),
        }
    }

    fn pop_bool(&mut self) -> IcnResult<bool> {
        match self.stack.pop() {
            Some(Value::Bool(b)) => Ok(b),
            _ => Err(IcnError::VM("Expected boolean value".into())),
        }
    }

    fn pop_list(&mut self) -> IcnResult<Vec<Value>> {
        match self.stack.pop() {
            Some(Value::List(l)) => Ok(l),
            _ => Err(IcnError::VM("Expected list value".into())),
        }
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

    pub fn execute_bytecode(&mut self, bytecode: Vec<Opcode>) -> IcnResult<()> {
        self.program = bytecode;
        self.pc = 0;
        self.run()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_language::compile_to_bytecode;

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
        assert!(vm.run().is_ok());
        assert_eq!(vm.stack, vec![Value::Float(16.0)]);
    }

    #[test]
    fn test_net_node_connect() {
        let program = vec![
            Opcode::Push(Value::String("node1".to_string())),
            Opcode::Push(Value::String("node2".to_string())),
            Opcode::NetNodeConnect,
        ];

        let mut vm = CoopVM::new(program);
        assert!(vm.run().is_ok());
    }

    #[test]
    fn test_create_proposal() {
        let program = vec![
            Opcode::Push(Value::String("Increase node count".to_string())),
            Opcode::Push(Value::String("Network Expansion".to_string())),
            Opcode::CreateProposal,
        ];

        let mut vm = CoopVM::new(program);
        assert!(vm.run().is_ok());
        assert_eq!(vm.stack, vec![Value::String("new_proposal_id".to_string())]);
    }

    #[test]
    fn test_execute_bytecode() {
        let source = r#"
            net-node-connect("node1", "node2")
            econ-currency-mint(100.0, "BasicNeeds")
        "#;
        let bytecode = compile_to_bytecode(source).unwrap();
        let mut vm = CoopVM::new(vec![]);
        assert!(vm.execute_bytecode(bytecode).is_ok());
        // Add assertions here to check the state of the VM
    }
}
