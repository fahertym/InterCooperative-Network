use icn_common::{IcnResult, IcnError};
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
    MorphemeCall(String, usize), // Name of the morpheme and number of arguments
    MorphemeReturn,
}

pub struct Morpheme {
    params: Vec<String>,
    body: Vec<Opcode>,
}

pub struct CoopVM {
    stack: Vec<Value>,
    memory: HashMap<String, Value>,
    program: Vec<Opcode>,
    pc: usize,
    call_stack: Vec<usize>,
    morphemes: HashMap<String, Morpheme>,
}

impl CoopVM {
    pub fn new(program: Vec<Opcode>) -> Self {
        CoopVM {
            stack: Vec::new(),
            memory: HashMap::new(),
            program,
            pc: 0,
            call_stack: Vec::new(),
            morphemes: HashMap::new(),
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
                self.call_stack.push(self.pc);
                // In a real implementation, you would look up the function and set the PC
            }
            Opcode::Return => {
                self.pc = self.call_stack.pop().ok_or(IcnError::VM("Return without call".into()))?;
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
            Opcode::MorphemeCall(name, arg_count) => self.morpheme_call(name, arg_count)?,
            Opcode::MorphemeReturn => {
                self.pc = self.call_stack.pop().ok_or(IcnError::VM("MorphemeReturn without call".into()))?;
            }
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

    fn pop_float(&mut self) -> IcnResult<f64> {
        match self.stack.pop() {
            Some(Value::Float(f)) => Ok(f),
            Some(Value::Int(i)) => Ok(i as f64),
            _ => Err(IcnError::VM("Expected float value".into())),
        }
    }

    fn pop_bool(&mut self) -> IcnResult<bool> {
        match self.stack.pop() {
            Some(Value::Bool(b)) => Ok(b),
            _ => Err(IcnError::VM("Expected boolean value".into())),
        }
    }

    fn pop_string(&mut self) -> IcnResult<String> {
        match self.stack.pop() {
            Some(Value::String(s)) => Ok(s),
            _ => Err(IcnError::VM("Expected string value".into())),
        }
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
        let amount = self.pop_float()?;
        let resource = self.pop_string()?;
        println!("Allocating {} units of resource {}", amount, resource);
        Ok(())
    }

    fn update_reputation(&mut self) -> IcnResult<()> {
        let change = self.pop_float()?;
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

    fn pop_list(&mut self) -> IcnResult<Vec<Value>> {
        match self.stack.pop() {
            Some(Value::List(l)) => Ok(l),
            _ => Err(IcnError::VM("Expected list value".into())),
        }
    }

    fn morpheme_call(&mut self, name: String, arg_count: usize) -> IcnResult<()> {
        let morpheme = self.morphemes.get(&name).ok_or(IcnError::VM(format!("Morpheme not found: {}", name)))?;
        let mut args = Vec::new();
        for _ in 0..arg_count {
            args.push(self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))?);
        }
        args.reverse();

        // Create a new scope for the morpheme
        let old_memory = std::mem::replace(&mut self.memory, HashMap::new());
        self.call_stack.push(self.pc);

        // Bind arguments to parameters
        for (param, arg) in morpheme.params.iter().zip(args) {
            self.memory.insert(param.clone(), arg);
        }

        // Set PC to the start of the morpheme body
        self.pc = self.program.len();
        self.program.extend_from_slice(&morpheme.body);

        // Push the old memory onto the call stack
        self.call_stack.push(self.pc);
        self.call_stack.push(old_memory.len() as usize);
        for (k, v) in old_memory {
            self.call_stack.push(self.program.len());
            self.program.push(Opcode::Push(Value::String(k)));
            self.program.push(Opcode::Push(v));
        }

        Ok(())
    }

    pub fn register_morpheme(&mut self, name: String, morpheme: Morpheme) {
        self.morphemes.insert(name, morpheme);
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
        assert!(vm.run().is_ok());
        assert_eq!(vm.stack, vec![Value::Float(16.0)]);
    }

    #[test]
    fn test_morpheme() {
        let transfer_morpheme = Morpheme {
            params: vec!["from".to_string(), "to".to_string(), "amount".to_string()],
            body: vec![
                Opcode::Load("amount".to_string()),
                Opcode::Load("from".to_string()),
                Opcode::UpdateReputation,
                Opcode::Load("amount".to_string()),
                Opcode::Load("to".to_string()),
                Opcode::UpdateReputation,
                Opcode::MorphemeReturn,
            ],
        };

        let mut vm = CoopVM::new(vec![
            Opcode::Push(Value::String("Alice".to_string())),
            Opcode::Push(Value::String("Bob".to_string())),
            Opcode::Push(Value::Float(100.0)),
            Opcode::MorphemeCall("transfer".to_string(), 3),
        ]);

        vm.register_morpheme("transfer".to_string(), transfer_morpheme);

        assert!(vm.run().is_ok());
        // The stack should be empty after the morpheme execution
        assert!(vm.stack.is_empty());
    }

    #[test]
    fn test_conditional_execution() {
        let program = vec![
            Opcode::Push(Value::Bool(true)),
            Opcode::JumpIfFalse(4),
            Opcode::Push(Value::String("Condition is true".to_string())),
            Opcode::Jump(5),
            Opcode::Push(Value::String("Condition is false".to_string())),
        ];

        let mut vm = CoopVM::new(program);
        assert!(vm.run().is_ok());
        assert_eq!(vm.stack, vec![Value::String("Condition is true".to_string())]);
    }

    #[test]
    fn test_cooperative_operations() {
        let program = vec![
            Opcode::Push(Value::String("Coop1".to_string())),
            Opcode::Push(Value::String("Alice".to_string())),
            Opcode::CoopMemberAdd,
            Opcode::Push(Value::String("Proposal 1".to_string())),
            Opcode::Push(Value::String("Increase node count".to_string())),
            Opcode::CreateProposal,
            Opcode::Push(Value::String("BasicNeeds".to_string())),
            Opcode::Push(Value::Float(100.0)),
            Opcode::EconCurrencyMint,
        ];

        let mut vm = CoopVM::new(program);
        assert!(vm.run().is_ok());
        assert_eq!(vm.stack, vec![Value::String("new_proposal_id".to_string())]);
    }

    #[test]
    fn test_error_handling() {
        let program = vec![
            Opcode::Push(Value::Int(5)),
            Opcode::Push(Value::String("Not a number".to_string())),
            Opcode::Add,
        ];

        let mut vm = CoopVM::new(program);
        let result = vm.run();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "VM error: Expected float value");
    }
}