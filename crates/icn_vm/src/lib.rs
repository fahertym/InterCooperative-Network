use icn_common::{IcnResult, IcnError, Opcode, Value};
use std::collections::HashMap;

pub struct CoopVM {
    stack: Vec<Value>,
    memory: HashMap<String, Value>,
    pc: usize,
}

impl CoopVM {
    pub fn new() -> Self {
        CoopVM {
            stack: Vec::new(),
            memory: HashMap::new(),
            pc: 0,
        }
    }

    pub fn execute(&mut self, program: Vec<Opcode>) -> IcnResult<()> {
        self.pc = 0;
        while self.pc < program.len() {
            match &program[self.pc] {
                Opcode::Push(value) => self.stack.push(value.clone()),
                Opcode::Pop => { self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))?; }
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
                    let value = self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))?;
                    self.memory.insert(name.clone(), value);
                }
                Opcode::Load(name) => {
                    let value = self.memory.get(name).ok_or(IcnError::VM("Variable not found".into()))?.clone();
                    self.stack.push(value);
                }
                Opcode::JumpIf(target) => {
                    let condition = self.pop_bool()?;
                    if condition {
                        self.pc = *target;
                        continue;
                    }
                    Opcode::Jump(target) => {
                        self.pc = *target;
                        continue;
                    }
                    Opcode::Call(_) => return Err(IcnError::VM("Function calls not implemented".into())),
                    Opcode::Return => break,
                    Opcode::NetNodeConnect => {
                        let node2 = self.pop_string()?;
                        let node1 = self.pop_string()?;
                        println!("Connecting nodes: {} and {}", node1, node2);
                    }
                    Opcode::ChainBlockCreate => {
                        let num_transactions = self.pop_int()?;
                        let mut transactions = Vec::new();
                        for _ in 0..num_transactions {
                            transactions.push(self.pop_string()?);
                        }
                        println!("Creating block with {} transactions", transactions.len());
                    }
                    Opcode::EconCurrencyMint => {
                        let currency_type = self.pop_string()?;
                        let amount = self.pop_float()?;
                        println!("Minting {} of currency type {}", amount, currency_type);
                    }
                    Opcode::GovProposalSubmit => {
                        let description = self.pop_string()?;
                        println!("Submitting proposal: {}", description);
                    }
                    Opcode::CoopMemberAdd => {
                        let member_id = self.pop_string()?;
                        let coop_id = self.pop_string()?;
                        println!("Adding member {} to cooperative {}", member_id, coop_id);
                    }
                    Opcode::CommEventOrganize => {
                        let event_details = self.pop_string()?;
                        println!("Organizing community event: {}", event_details);
                    }
                    Opcode::VoteOnProposal => {
                        let vote = self.pop_bool()?;
                        let proposal_id = self.pop_string()?;
                        println!("Voting {} on proposal {}", if vote { "Yes" } else { "No" }, proposal_id);
                    }
                    Opcode::AllocateResource => {
                        let amount = self.pop_int()?;
                        let resource = self.pop_string()?;
                        println!("Allocating {} units of resource {}", amount, resource);
                    }
                    Opcode::UpdateReputation => {
                        let change = self.pop_int()?;
                        let address = self.pop_string()?;
                        println!("Updating reputation of {} by {}", address, change);
                    }
                    Opcode::CreateProposal => {
                        let description = self.pop_string()?;
                        let title = self.pop_string()?;
                        println!("Creating proposal: {} - {}", title, description);
                        self.stack.push(Value::String("new_proposal_id".to_string()));
                    }
                    Opcode::GetProposalStatus => {
                        let proposal_id = self.pop_string()?;
                        println!("Getting status of proposal: {}", proposal_id);
                        self.stack.push(Value::String("Active".to_string()));
                    }
                    Opcode::EmitEvent(event_name) => {
                        let event_data = self.pop_string()?;
                        println!("Emitting event {}: {}", event_name, event_data);
                    }
                }
                self.pc += 1;
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
    
        fn pop_float(&mut self) -> IcnResult<f64> {
            match self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))? {
                Value::Float(f) => Ok(f),
                Value::Int(i) => Ok(i as f64),
                _ => Err(IcnError::VM("Expected float value".into())),
            }
        }
    
        fn pop_bool(&mut self) -> IcnResult<bool> {
            match self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))? {
                Value::Bool(b) => Ok(b),
                _ => Err(IcnError::VM("Expected boolean value".into())),
            }
        }
    
        fn pop_string(&mut self) -> IcnResult<String> {
            match self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))? {
                Value::String(s) => Ok(s),
                _ => Err(IcnError::VM("Expected string value".into())),
            }
        }
    
        fn pop_int(&mut self) -> IcnResult<i64> {
            match self.stack.pop().ok_or(IcnError::VM("Stack underflow".into()))? {
                Value::Int(i) => Ok(i),
                _ => Err(IcnError::VM("Expected integer value".into())),
            }
        }
    }
    
    #[cfg(test)]
    mod tests {
        use super::*;
    
        #[test]
        fn test_basic_operations() {
            let mut vm = CoopVM::new();
            let program = vec![
                Opcode::Push(Value::Float(5.0)),
                Opcode::Push(Value::Float(3.0)),
                Opcode::Add,
                Opcode::Push(Value::Float(2.0)),
                Opcode::Mul,
            ];
    
            assert!(vm.execute(program).is_ok());
            assert_eq!(vm.stack, vec![Value::Float(16.0)]);
        }
    
        #[test]
        fn test_conditional_execution() {
            let mut vm = CoopVM::new();
            let program = vec![
                Opcode::Push(Value::Bool(true)),
                Opcode::JumpIf(4),
                Opcode::Push(Value::String("Condition is false".to_string())),
                Opcode::Jump(5),
                Opcode::Push(Value::String("Condition is true".to_string())),
            ];
    
            assert!(vm.execute(program).is_ok());
            assert_eq!(vm.stack, vec![Value::String("Condition is true".to_string())]);
        }
    
        #[test]
        fn test_cooperative_operations() {
            let mut vm = CoopVM::new();
            let program = vec![
                Opcode::Push(Value::String("Proposal 1".to_string())),
                Opcode::Push(Value::String("New community project".to_string())),
                Opcode::CreateProposal,
                Opcode::Push(Value::String("proposal_1".to_string())),
                Opcode::Push(Value::Bool(true)),
                Opcode::VoteOnProposal,
                Opcode::Push(Value::String("computing_power".to_string())),
                Opcode::Push(Value::Int(100)),
                Opcode::AllocateResource,
                Opcode::Push(Value::String("user1".to_string())),
                Opcode::Push(Value::Int(5)),
                Opcode::UpdateReputation,
                Opcode::Push(Value::String("proposal_1".to_string())),
                Opcode::GetProposalStatus,
        ];

        assert!(vm.execute(program).is_ok());
        assert_eq!(vm.stack, vec![Value::String("new_proposal_id".to_string()), Value::String("Active".to_string())]);
    }

    #[test]
    fn test_error_handling() {
        let mut vm = CoopVM::new();
        let program = vec![
            Opcode::Push(Value::Int(5)),
            Opcode::Push(Value::String("Not a number".to_string())),
            Opcode::Add,
        ];

        let result = vm.execute(program);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IcnError::VM(_)));
    }

    #[test]
    fn test_store_and_load() {
        let mut vm = CoopVM::new();
        let program = vec![
            Opcode::Push(Value::Int(42)),
            Opcode::Store("x".to_string()),
            Opcode::Load("x".to_string()),
            Opcode::Push(Value::Int(10)),
            Opcode::Add,
        ];

        assert!(vm.execute(program).is_ok());
        assert_eq!(vm.stack, vec![Value::Float(52.0)]);
    }

    #[test]
    fn test_network_operations() {
        let mut vm = CoopVM::new();
        let program = vec![
            Opcode::Push(Value::String("node1".to_string())),
            Opcode::Push(Value::String("node2".to_string())),
            Opcode::NetNodeConnect,
            Opcode::Push(Value::String("tx1".to_string())),
            Opcode::Push(Value::String("tx2".to_string())),
            Opcode::Push(Value::Int(2)),
            Opcode::ChainBlockCreate,
        ];

        assert!(vm.execute(program).is_ok());
        // The stack should be empty after these operations
        assert!(vm.stack.is_empty());
    }

    #[test]
    fn test_economic_operations() {
        let mut vm = CoopVM::new();
        let program = vec![
            Opcode::Push(Value::Float(100.0)),
            Opcode::Push(Value::String("BasicNeeds".to_string())),
            Opcode::EconCurrencyMint,
        ];

        assert!(vm.execute(program).is_ok());
        // The stack should be empty after these operations
        assert!(vm.stack.is_empty());
    }

    #[test]
    fn test_governance_operations() {
        let mut vm = CoopVM::new();
        let program = vec![
            Opcode::Push(Value::String("Increase node count".to_string())),
            Opcode::GovProposalSubmit,
            Opcode::Push(Value::String("coop1".to_string())),
            Opcode::Push(Value::String("member1".to_string())),
            Opcode::CoopMemberAdd,
        ];

        assert!(vm.execute(program).is_ok());
        // The stack should be empty after these operations
        assert!(vm.stack.is_empty());
    }

    #[test]
    fn test_community_operations() {
        let mut vm = CoopVM::new();
        let program = vec![
            Opcode::Push(Value::String("Community meetup on Saturday".to_string())),
            Opcode::CommEventOrganize,
            Opcode::Push(Value::String("NewMember".to_string())),
            Opcode::Push(Value::String("Alice joined the network".to_string())),
            Opcode::EmitEvent("NewMember".to_string()),
        ];

        assert!(vm.execute(program).is_ok());
        // The stack should be empty after these operations
        assert!(vm.stack.is_empty());
    }
    }
}
