use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    List(Vec<Value>),
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
    CreateList,
    AppendList,
    GetListItem,
    SetListItem,
}

#[derive(Error, Debug)]
pub enum VMError {
    #[error("Stack underflow")]
    StackUnderflow,
    #[error("Type mismatch: expected {0}, found {1}")]
    TypeMismatch(String, String),
    #[error("Unknown variable: {0}")]
    UnknownVariable(String),
    #[error("Unknown function: {0}")]
    UnknownFunction(String),
    #[error("Invalid jump target")]
    InvalidJumpTarget,
    #[error("Return without call")]
    ReturnWithoutCall,
    #[error("List index out of bounds")]
    ListIndexOutOfBounds,
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

    pub fn run(&mut self) -> Result<(), VMError> {
        while self.pc < self.program.len() {
            self.execute_instruction()?;
            self.pc += 1;
        }
        Ok(())
    }

    fn execute_instruction(&mut self) -> Result<(), VMError> {
        let instruction = &self.program[self.pc];
        match instruction {
            Opcode::Push(value) => self.stack.push(value.clone()),
            Opcode::Pop => { self.stack.pop().ok_or(VMError::StackUnderflow)?; }
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
                let value = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                self.memory.insert(name.clone(), value);
            }
            Opcode::Load(name) => {
                let value = self.memory.get(name).ok_or(VMError::UnknownVariable(name.clone()))?.clone();
                self.stack.push(value);
            }
            Opcode::Call(func_name) => {
                let func_pc = *self.functions.get(func_name).ok_or(VMError::UnknownFunction(func_name.clone()))?;
                self.call_stack.push(self.pc);
                self.pc = func_pc;
            }
            Opcode::Return => {
                self.pc = self.call_stack.pop().ok_or(VMError::ReturnWithoutCall)?;
            }
            Opcode::JumpIf(target) => {
                let condition = self.pop_bool()?;
                if condition {
                    self.pc = *target;
                }
            }
            Opcode::Jump(target) => {
                self.pc = *target;
            }
            Opcode::CreateList => {
                self.stack.push(Value::List(Vec::new()));
            }
            Opcode::AppendList => {
                let value = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                if let Some(Value::List(list)) = self.stack.last_mut() {
                    list.push(value);
                } else {
                    return Err(VMError::TypeMismatch("List".to_string(), "Non-List".to_string()));
                }
            }
            Opcode::GetListItem => {
                let index = self.pop_int()?;
                if let Some(Value::List(list)) = self.stack.pop() {
                    let item = list.get(index as usize).ok_or(VMError::ListIndexOutOfBounds)?.clone();
                    self.stack.push(item);
                } else {
                    return Err(VMError::TypeMismatch("List".to_string(), "Non-List".to_string()));
                }
            }
            Opcode::SetListItem => {
                let value = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                let index = self.pop_int()?;
                if let Some(Value::List(list)) = self.stack.last_mut() {
                    if (index as usize) < list.len() {
                        list[index as usize] = value;
                    } else {
                        return Err(VMError::ListIndexOutOfBounds);
                    }
                } else {
                    return Err(VMError::TypeMismatch("List".to_string(), "Non-List".to_string()));
                }
            }
        }
        Ok(())
    }

    fn binary_op<F>(&mut self, op: F) -> Result<(), VMError>
    where
        F: Fn(i64, i64) -> i64,
    {
        let b = self.pop_int()?;
        let a = self.pop_int()?;
        self.stack.push(Value::Int(op(a, b)));
        Ok(())
    }

    fn compare_op<F>(&mut self, op: F) -> Result<(), VMError>
    where
        F: Fn(&Value, &Value) -> bool,
    {
        let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
        let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
        self.stack.push(Value::Bool(op(&a, &b)));
        Ok(())
    }

    fn logic_op<F>(&mut self, op: F) -> Result<(), VMError>
    where
        F: Fn(bool, bool) -> bool,
    {
        let b = self.pop_bool()?;
        let a = self.pop_bool()?;
        self.stack.push(Value::Bool(op(a, b)));
        Ok(())
    }

    fn pop_int(&mut self) -> Result<i64, VMError> {
        match self.stack.pop().ok_or(VMError::StackUnderflow)? {
            Value::Int(i) => Ok(i),
            v => Err(VMError::TypeMismatch("Int".to_string(), format!("{:?}", v))),
        }
    }

    fn pop_bool(&mut self) -> Result<bool, VMError> {
        match self.stack.pop().ok_or(VMError::StackUnderflow)? {
            Value::Bool(b) => Ok(b),
            v => Err(VMError::TypeMismatch("Bool".to_string(), format!("{:?}", v))),
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

    pub fn set_memory(&mut self, name: String, value: Value) {
        self.memory.insert(name, value);
    }

    pub fn load_program(&mut self, program: Vec<Opcode>) {
        self.program = program;
        self.pc = 0;
        self.stack.clear();
        self.memory.clear();
        self.call_stack.clear();
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

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_code(&self) -> &Vec<Opcode> {
        &self.code
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

    pub fn deploy_contract(&mut self, contract: SmartContract) -> Result<(), VMError> {
        if self.contracts.contains_key(&contract.name) {
            return Err(VMError::UnknownFunction(format!("Contract {} already exists", contract.name)));
        }
        self.contracts.insert(contract.name.clone(), contract);
        Ok(())
    }

    pub fn execute_contract(&mut self, contract_name: &str, input: &[Value]) -> Result<Vec<Value>, VMError> {
        let contract = self.contracts.get(contract_name)
            .ok_or_else(|| VMError::UnknownFunction(format!("Contract {} not found", contract_name)))?;
        
        self.vm.load_program(contract.code.clone());
        
        // Set up input parameters in VM memory
        for (i, value) in input.iter().enumerate() {
            self.vm.set_memory(format!("input_{}", i), value.clone());
        }

        self.vm.run()?;

        // Collect output from VM stack
        Ok(self.vm.get_stack().clone())
    }

    pub fn get_contract(&self, name: &str) -> Option<&SmartContract> {
        self.contracts.get(name)
    }

    pub fn list_contracts(&self) -> Vec<String> {
        self.contracts.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_contract() {
        let mut manager = ContractManager::new();

        let contract = SmartContract::new(
            "SimpleAddition".to_string(),
            vec![
                Opcode::Load("input_0".to_string()),
                Opcode::Load("input_1".to_string()),
                Opcode::Add,
            ],
        );

        manager.deploy_contract(contract).unwrap();

        let result = manager.execute_contract(
            "SimpleAddition",
            &[Value::Int(5), Value::Int(3)],
        ).unwrap();

        assert_eq!(result, vec![Value::Int(8)]);
    }

    #[test]
    fn test_complex_contract() {
        let mut manager = ContractManager::new();

        let contract = SmartContract::new(
            "ComplexOperations".to_string(),
            vec![
                Opcode::CreateList,
                Opcode::Load("input_0".to_string()),
                Opcode::AppendList,
                Opcode::Load("input_1".to_string()),
                Opcode::AppendList,
                Opcode::Push(Value::Int(0)),
                Opcode::GetListItem,
                Opcode::Push(Value::Int(1)),
                Opcode::GetListItem,
                Opcode::Mul,
            ],
        );

        manager.deploy_contract(contract).unwrap();

        let result = manager.execute_contract(
            "ComplexOperations",
            &[Value::Int(5), Value::Int(3)],
        ).unwrap();

        assert_eq!(result, vec![Value::Int(15)]);
    }
}