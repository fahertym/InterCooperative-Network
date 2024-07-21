use icn_common::{IcnResult, IcnError};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
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

    pub fn get_stack(&self) -> &Vec<Value> {
        &self.stack
    }

    pub fn get_memory(&self) -> &HashMap<String, Value> {
        &self.memory
    }

    pub fn load_program(&mut self, program: Vec<Opcode>) {
        self.program = program;
        self.pc = 0;
        self.stack.clear();
        self.memory.clear();
    }
}

pub struct SmartContract {
    pub name: String,
    pub code: Vec<Opcode>,
}

impl SmartContract {
    pub fn new(name: String, code: Vec<Opcode>) -> Self {
        SmartContract { name, code }
    }
}

pub struct ContractManager {
    contracts: HashMap<String, SmartContract>,
    vm: CoopVM,
}

impl ContractManager {
    pub fn new() -> Self {
        ContractManager {
            contracts: HashMap::new(),
            vm: CoopVM::new(Vec::new()),
        }
    }

    pub fn deploy_contract(&mut self, contract: SmartContract) -> IcnResult<()> {
        if self.contracts.contains_key(&contract.name) {
            return Err(IcnError::VM(format!("Contract {} already exists", contract.name)));
        }
        self.contracts.insert(contract.name.clone(), contract);
        Ok(())
    }

    pub fn execute_contract(&mut self, contract_name: &str) -> IcnResult<()> {
        let contract = self.contracts.get(contract_name)
            .ok_or_else(|| IcnError::VM(format!("Contract {} not found", contract_name)))?;
        self.vm.load_program(contract.code.clone());
        self.vm.run()
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

        assert_eq!(vm.get_stack(), &vec![Value::Int(16)]);
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

        assert_eq!(vm.get_stack(), &vec![Value::Int(5)]);
    }

    #[test]
    fn test_contract_manager() {
        let mut manager = ContractManager::new();

        let contract1 = SmartContract::new(
            "SimpleAddition".to_string(),
            vec![
                Opcode::Push(Value::Int(5)),
                Opcode::Push(Value::Int(3)),
                Opcode::Add,
            ],
        );

        manager.deploy_contract(contract1).unwrap();
        manager.execute_contract("SimpleAddition").unwrap();

        assert_eq!(manager.vm.get_stack(), &vec![Value::Int(8)]);

        // Test deploying a contract with the same name
        let contract2 = SmartContract::new(
            "SimpleAddition".to_string(),
            vec![
                Opcode::Push(Value::Int(1)),
                Opcode::Push(Value::Int(1)),
                Opcode::Add,
            ],
        );

        assert!(manager.deploy_contract(contract2).is_err());

        // Test executing a non-existent contract
        assert!(manager.execute_contract("NonExistentContract").is_err());
    }
}
