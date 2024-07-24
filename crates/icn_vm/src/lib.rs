use icn_common::{IcnResult, IcnError};
use std::collections::HashMap;
use icn_blockchain::Block;
use icn_consensus::PoCConsensus;
use icn_currency::CurrencySystem;
use icn_governance::GovernanceSystem;

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
    MorphemicCall(String, usize),
    Return,
}

pub struct CoopVM {
    stack: Vec<Value>,
    memory: HashMap<String, Value>,
    program: Vec<Opcode>,
    pc: usize,
    blockchain: Box<dyn Block>,
    consensus: Box<dyn PoCConsensus>,
    currency_system: Box<dyn CurrencySystem>,
    governance: Box<dyn GovernanceSystem>,
}

impl CoopVM {
    pub fn new(
        program: Vec<Opcode>,
        blockchain: Box<dyn Block>,
        consensus: Box<dyn PoCConsensus>,
        currency_system: Box<dyn CurrencySystem>,
        governance: Box<dyn GovernanceSystem>,
    ) -> Self {
        CoopVM {
            stack: Vec::new(),
            memory: HashMap::new(),
            program,
            pc: 0,
            blockchain,
            consensus,
            currency_system,
            governance,
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
            Opcode::MorphemicCall(function, arg_count) => {
                self.morphemic_call(&function, arg_count)?;
            }
            Opcode::Return => {
                // For simplicity, we'll just stop execution
                self.pc = self.program.len();
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

    fn morphemic_call(&mut self, function: &str, arg_count: usize) -> IcnResult<()> {
        let mut args = Vec::new();
        for _ in 0..arg_count {
            args.push(self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))?);
        }
        args.reverse();

        match function {
            "net-node-connect" => self.net_node_connect(args),
            "chain-block-validate" => self.chain_block_validate(args),
            "econ-currency-mint" => self.econ_currency_mint(args),
            "gov-proposal-submit" => self.gov_proposal_submit(args),
            "coop-member-add" => self.coop_member_add(args),
            "comm-event-organize" => self.comm_event_organize(args),
            _ => Err(IcnError::VM(format!("Unknown morphemic function: {}", function))),
        }
    }

    fn net_node_connect(&mut self, args: Vec<Value>) -> IcnResult<()> {
        if args.len() != 2 {
            return Err(IcnError::VM("net-node-connect expects 2 arguments".into()));
        }
        let node1 = self.value_to_string(&args[0])?;
        let node2 = self.value_to_string(&args[1])?;
        
        // Here you would implement the actual node connection logic
        println!("Connecting nodes: {} and {}", node1, node2);
        Ok(())
    }

    fn chain_block_validate(&mut self, args: Vec<Value>) -> IcnResult<()> {
        if args.len() != 1 {
            return Err(IcnError::VM("chain-block-validate expects 1 argument".into()));
        }
        let block_id = self.value_to_string(&args[0])?;
        
        // Here you would implement the actual block validation logic
        let is_valid = self.blockchain.validate_block(&block_id)?;
        self.stack.push(Value::Bool(is_valid));
        Ok(())
    }

    fn econ_currency_mint(&mut self, args: Vec<Value>) -> IcnResult<()> {
        if args.len() != 2 {
            return Err(IcnError::VM("econ-currency-mint expects 2 arguments".into()));
        }
        let amount = self.value_to_number(&args[0])?;
        let currency_type = self.value_to_string(&args[1])?;
        
        // Here you would implement the actual currency minting logic
        self.currency_system.mint(&currency_type, amount)?;
        Ok(())
    }

    fn gov_proposal_submit(&mut self, args: Vec<Value>) -> IcnResult<()> {
        if args.len() != 1 {
            return Err(IcnError::VM("gov-proposal-submit expects 1 argument".into()));
        }
        let proposal = self.value_to_string(&args[0])?;
        
        // Here you would implement the actual proposal submission logic
        let proposal_id = self.governance.submit_proposal(&proposal)?;
        self.stack.push(Value::String(proposal_id));
        Ok(())
    }

    fn coop_member_add(&mut self, args: Vec<Value>) -> IcnResult<()> {
        if args.len() != 2 {
            return Err(IcnError::VM("coop-member-add expects 2 arguments".into()));
        }
        let coop_id = self.value_to_string(&args[0])?;
        let member_id = self.value_to_string(&args[1])?;
        
        // Here you would implement the actual member addition logic
        // For this example, we'll just print the action
        println!("Adding member {} to cooperative {}", member_id, coop_id);
        Ok(())
    }

    fn comm_event_organize(&mut self, args: Vec<Value>) -> IcnResult<()> {
        if args.len() != 1 {
            return Err(IcnError::VM("comm-event-organize expects 1 argument".into()));
        }
        let event_details = self.value_to_string(&args[0])?;
        
        // Here you would implement the actual event organization logic
        // For this example, we'll just print the action
        println!("Organizing community event: {}", event_details);
        Ok(())
    }

    fn value_to_string(&self, value: &Value) -> IcnResult<String> {
        match value {
            Value::String(s) => Ok(s.clone()),
            _ => Err(IcnError::VM("Expected string value".into())),
        }
    }

    fn value_to_number(&self, value: &Value) -> IcnResult<f64> {
        match value {
            Value::Int(i) => Ok(*i as f64),
            Value::Float(f) => Ok(*f),
            _ => Err(IcnError::VM("Expected numeric value".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct MockBlock;
    impl Block for MockBlock {
        fn validate_block(&self, _block_id: &str) -> IcnResult<bool> {
            Ok(true)
        }
    }

    struct MockConsensus;
    impl PoCConsensus for MockConsensus {}

    struct MockCurrencySystem {
        minted: Rc<RefCell<HashMap<String, f64>>>,
    }
    impl CurrencySystem for MockCurrencySystem {
        fn mint(&self, currency_type: &str, amount: f64) -> IcnResult<()> {
            let mut minted = self.minted.borrow_mut();
            *minted.entry(currency_type.to_string()).or_default() += amount;
            Ok(())
        }
    }

    struct MockGovernance {
        proposals: Rc<RefCell<Vec<String>>>,
    }
    impl GovernanceSystem for MockGovernance {
        fn submit_proposal(&self, proposal: &str) -> IcnResult<String> {
            let mut proposals = self.proposals.borrow_mut();
            proposals.push(proposal.to_string());
            Ok(format!("proposal_{}", proposals.len()))
        }
    }

    #[test]
    fn test_morphemic_operations() {
        let program = vec![
            Opcode::Push(Value::String("node1".to_string())),
            Opcode::Push(Value::String("node2".to_string())),
            Opcode::MorphemicCall("net-node-connect".to_string(), 2),
            Opcode::Push(Value::Int(100)),
            Opcode::Push(Value::String("BasicNeeds".to_string())),
            Opcode::MorphemicCall("econ-currency-mint".to_string(), 2),
            Opcode::Push(Value::String("Increase node count".to_string())),
            Opcode::MorphemicCall("gov-proposal-submit".to_string(), 1),
        ];

        let minted = Rc::new(RefCell::new(HashMap::new()));
        let proposals = Rc::new(RefCell::new(Vec::new()));

        let mut vm = CoopVM::new(
            program,
            Box::new(MockBlock),
            Box::new(MockConsensus),
            Box::new(MockCurrencySystem { minted: Rc::clone(&minted) }),
            Box::new(MockGovernance { proposals: Rc::clone(&proposals) }),
        );

        assert!(vm.run().is_ok());

        assert_eq!(*minted.borrow().get("BasicNeeds").unwrap(), 100.0);
        assert_eq!(proposals.borrow().len(), 1);
        assert_eq!(proposals.borrow()[0], "Increase node count");

        // Check if the proposal ID is on the stack
        match vm.stack.pop() {
            Some(Value::String(id)) => assert_eq!(id, "proposal_1"),
            _ => panic!("Expected proposal ID on the stack"),
        }
    }

    #[test]
    fn test_control_flow() {
        let program = vec![
            Opcode::Push(Value::Int(10)),
            Opcode::Store("x".to_string()),
            Opcode::Push(Value::Int(0)),
            Opcode::Store("sum".to_string()),
            // Start of while loop
            Opcode::Load("x".to_string()),
            Opcode::Push(Value::Int(0)),
            Opcode::Gt,
            Opcode::JumpIfFalse(17), // Jump to after the loop if x <= 0
            Opcode::Load("sum".to_string()),
            Opcode::Load("x".to_string()),
            Opcode::Add,
            Opcode::Store("sum".to_string()),
            Opcode::Load("x".to_string()),
            Opcode::Push(Value::Int(1)),
            Opcode::Sub,
            Opcode::Store("x".to_string()),
            Opcode::Jump(4), // Jump back to the start of the loop
            // End of while loop
            Opcode::Load("sum".to_string()),
        ];

        let mut vm = CoopVM::new(
            program,
            Box::new(MockBlock),
            Box::new(MockConsensus),
            Box::new(MockCurrencySystem { minted: Rc::new(RefCell::new(HashMap::new())) }),
            Box::new(MockGovernance { proposals: Rc::new(RefCell::new(Vec::new())) }),
        );

        assert!(vm.run().is_ok());

        // Check if the sum is correct (should be 55)
        match vm.stack.pop() {
            Some(Value::Float(sum)) => assert_eq!(sum, 55.0),
            _ => panic!("Expected sum on the stack"),
        }
    }
}