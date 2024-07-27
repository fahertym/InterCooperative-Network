use pest::Parser;
use pest_derive::Parser;
use icn_common::{IcnResult, IcnError, CurrencyType};
use icn_vm::{CoopVM, Opcode, Value};
use std::collections::HashMap;
use std::fmt;

// Define the parser for our smart contract language
#[derive(Parser)]
#[grammar = "contract.pest"]
struct ContractParser;

// Enum to represent different types of smart contracts
#[derive(Debug, Clone)]
pub enum SmartContractType {
    AssetTransfer,
    VotingSystem,
    ReputationManagement,
    ResourceAllocation,
    CustomLogic,
}

// Struct to represent a compiled smart contract
#[derive(Debug)]
pub struct CompiledContract {
    contract_type: SmartContractType,
    bytecode: Vec<Opcode>,
    abi: ContractABI,
}

// Struct to represent the ABI (Application Binary Interface) of a smart contract
#[derive(Debug)]
pub struct ContractABI {
    functions: Vec<ContractFunction>,
    events: Vec<ContractEvent>,
}

// Struct to represent a function in the smart contract's ABI
#[derive(Debug)]
pub struct ContractFunction {
    name: String,
    inputs: Vec<ContractParameter>,
    outputs: Vec<ContractParameter>,
}

// Struct to represent an event in the smart contract's ABI
#[derive(Debug)]
pub struct ContractEvent {
    name: String,
    parameters: Vec<ContractParameter>,
}

// Struct to represent a parameter in a function or event
#[derive(Debug)]
pub struct ContractParameter {
    name: String,
    param_type: ContractValueType,
}

// Enum to represent the possible value types in a smart contract
#[derive(Debug, Clone)]
pub enum ContractValueType {
    Integer,
    Float,
    Boolean,
    String,
    Address,
    List(Box<ContractValueType>),
    Map(Box<ContractValueType>, Box<ContractValueType>),
}

impl fmt::Display for ContractValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractValueType::Integer => write!(f, "Integer"),
            ContractValueType::Float => write!(f, "Float"),
            ContractValueType::Boolean => write!(f, "Boolean"),
            ContractValueType::String => write!(f, "String"),
            ContractValueType::Address => write!(f, "Address"),
            ContractValueType::List(inner) => write!(f, "List<{}>", inner),
            ContractValueType::Map(key, value) => write!(f, "Map<{}, {}>", key, value),
        }
    }
}

// The main compiler for our natural language smart contracts
pub struct NaturalLanguageCompiler;

impl NaturalLanguageCompiler {
    // Compile a natural language contract into bytecode
    pub fn compile(input: &str) -> IcnResult<CompiledContract> {
        let pairs = ContractParser::parse(Rule::contract, input)
            .map_err(|e| IcnError::SmartContract(format!("Parsing error: {}", e)))?;

        let mut bytecode = Vec::new();
        let mut abi = ContractABI {
            functions: Vec::new(),
            events: Vec::new(),
        };
        let mut contract_type = SmartContractType::CustomLogic;

        for pair in pairs {
            match pair.as_rule() {
                Rule::contract_type => {
                    contract_type = Self::parse_contract_type(pair.into_inner().next().unwrap().as_str())?;
                }
                Rule::function_definition => {
                    let (func_bytecode, func_abi) = Self::compile_function(pair)?;
                    bytecode.extend(func_bytecode);
                    abi.functions.push(func_abi);
                }
                Rule::statement => {
                    bytecode.extend(Self::compile_statement(pair)?);
                }
                Rule::event_definition => {
                    abi.events.push(Self::compile_event(pair)?);
                }
                _ => {}
            }
        }

        Ok(CompiledContract {
            contract_type,
            bytecode,
            abi,
        })
    }

    // Parse the contract type from the input
    fn parse_contract_type(type_str: &str) -> IcnResult<SmartContractType> {
        match type_str {
            "AssetTransfer" => Ok(SmartContractType::AssetTransfer),
            "VotingSystem" => Ok(SmartContractType::VotingSystem),
            "ReputationManagement" => Ok(SmartContractType::ReputationManagement),
            "ResourceAllocation" => Ok(SmartContractType::ResourceAllocation),
            "CustomLogic" => Ok(SmartContractType::CustomLogic),
            _ => Err(IcnError::SmartContract(format!("Unknown contract type: {}", type_str))),
        }
    }

    // Compile a function definition
    fn compile_function(pair: pest::iterators::Pair<Rule>) -> IcnResult<(Vec<Opcode>, ContractFunction)> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let params = inner.next().unwrap();
        let body = inner.next().unwrap();

        let mut inputs = Vec::new();
        let mut param_bytecode = Vec::new();
        for param in params.into_inner() {
            let mut param_inner = param.into_inner();
            let param_name = param_inner.next().unwrap().as_str().to_string();
            let param_type = Self::parse_type(param_inner.next().unwrap().as_str())?;
            inputs.push(ContractParameter {
                name: param_name.clone(),
                param_type: param_type.clone(),
            });
            param_bytecode.push(Opcode::Store(param_name));
        }

        let mut body_bytecode = Self::compile_statement(body)?;
        let mut bytecode = param_bytecode;
        bytecode.extend(body_bytecode);
        bytecode.push(Opcode::Return);

        let func_abi = ContractFunction {
            name,
            inputs,
            outputs: vec![],  // For simplicity, we're not handling return types yet
        };

        Ok((bytecode, func_abi))
    }

    // Compile a single statement
    fn compile_statement(pair: pest::iterators::Pair<Rule>) -> IcnResult<Vec<Opcode>> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::assignment => Self::compile_assignment(inner),
            Rule::if_statement => Self::compile_if_statement(inner),
            Rule::while_loop => Self::compile_while_loop(inner),
            Rule::function_call => Self::compile_function_call(inner),
            Rule::return_statement => Self::compile_return_statement(inner),
            _ => Err(IcnError::SmartContract("Unknown statement type".into())),
        }
    }

    // Compile an assignment statement
    fn compile_assignment(pair: pest::iterators::Pair<Rule>) -> IcnResult<Vec<Opcode>> {
        let mut inner = pair.into_inner();
        let variable = inner.next().unwrap().as_str();
        let value = inner.next().unwrap();

        let mut opcodes = Self::compile_expression(value)?;
        opcodes.push(Opcode::Store(variable.to_string()));
        Ok(opcodes)
    }

    // Compile an if statement
    fn compile_if_statement(pair: pest::iterators::Pair<Rule>) -> IcnResult<Vec<Opcode>> {
        let mut inner = pair.into_inner();
        let condition = inner.next().unwrap();
        let true_branch = inner.next().unwrap();
        let false_branch = inner.next();

        let mut opcodes = Self::compile_expression(condition)?;
        let true_opcodes = Self::compile_statement(true_branch)?;
        let false_opcodes = false_branch.map(Self::compile_statement).transpose()?;

        opcodes.push(Opcode::JumpIf(true_opcodes.len() as usize + 1));
        opcodes.extend(true_opcodes);
        if let Some(false_opcodes) = false_opcodes {
            opcodes.push(Opcode::Jump(false_opcodes.len() as usize));
            opcodes.extend(false_opcodes);
        }

        Ok(opcodes)
    }

    // Compile a while loop
    fn compile_while_loop(pair: pest::iterators::Pair<Rule>) -> IcnResult<Vec<Opcode>> {
        let mut inner = pair.into_inner();
        let condition = inner.next().unwrap();
        let body = inner.next().unwrap();

        let condition_opcodes = Self::compile_expression(condition)?;
        let body_opcodes = Self::compile_statement(body)?;

        let mut opcodes = Vec::new();
        opcodes.extend(condition_opcodes.clone());
        opcodes.push(Opcode::JumpIf(body_opcodes.len() as usize + 2));
        opcodes.extend(body_opcodes);
        opcodes.extend(condition_opcodes);
        opcodes.push(Opcode::JumpIf(usize::MAX - (opcodes.len() - 1)));

        Ok(opcodes)
    }

    // Compile a function call
    fn compile_function_call(pair: pest::iterators::Pair<Rule>) -> IcnResult<Vec<Opcode>> {
        let mut inner = pair.into_inner();
        let function_name = inner.next().unwrap().as_str();
        let arguments = inner.next().unwrap().into_inner();

        let mut opcodes = Vec::new();
        for arg in arguments {
            opcodes.extend(Self::compile_expression(arg)?);
        }
        opcodes.push(Opcode::Call(function_name.to_string()));
        Ok(opcodes)
    }

    // Compile a return statement
    fn compile_return_statement(pair: pest::iterators::Pair<Rule>) -> IcnResult<Vec<Opcode>> {
        let mut inner = pair.into_inner();
        let expression = inner.next().unwrap();

        let mut opcodes = Self::compile_expression(expression)?;
        opcodes.push(Opcode::Return);
        Ok(opcodes)
    }

    // Compile an expression
    fn compile_expression(pair: pest::iterators::Pair<Rule>) -> IcnResult<Vec<Opcode>> {
        match pair.as_rule() {
            Rule::number => Ok(vec![Opcode::Push(Value::Int(pair.as_str().parse().unwrap()))]),
            Rule::string => Ok(vec![Opcode::Push(Value::String(pair.into_inner().next().unwrap().as_str().to_string()))]),
            Rule::boolean => Ok(vec![Opcode::Push(Value::Bool(pair.as_str().parse().unwrap()))]),
            Rule::variable => Ok(vec![Opcode::Load(pair.as_str().to_string())]),
            Rule::expression => {
                let mut inner = pair.into_inner();
                let left = inner.next().unwrap();
                let operator = inner.next();
                let right = inner.next();

                let mut opcodes = Self::compile_expression(left)?;
                if let (Some(op), Some(right)) = (operator, right) {
                    opcodes.extend(Self::compile_expression(right)?);
                    opcodes.push(match op.as_str() {
                        "+" => Opcode::Add,
                        "-" => Opcode::Sub,
                        "*" => Opcode::Mul,
                        "/" => Opcode::Div,
                        "==" => Opcode::Eq,
                        "!=" => Opcode::Neq,
                        "<" => Opcode::Lt,
                        "<=" => Opcode::Lte,
                        ">" => Opcode::Gt,
                        ">=" => Opcode::Gte,
                        "and" => Opcode::And,
                        "or" => Opcode::Or,
                        _ => return Err(IcnError::SmartContract(format!("Unknown operator: {}", op.as_str()))),
                    });
                }
                Ok(opcodes)
            }
            _ => Err(IcnError::SmartContract("Unknown expression type".into())),
        }
    }

    // Compile an event definition
    fn compile_event(pair: pest::iterators::Pair<Rule>) -> IcnResult<ContractEvent> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let params = inner.next().unwrap();

        let mut parameters = Vec::new();
        for param in params.into_inner() {
            let mut param_inner = param.into_inner();
            let param_name = param_inner.next().unwrap().as_str().to_string();
            let param_type = Self::parse_type(param_inner.next().unwrap().as_str())?;
            parameters.push(ContractParameter {
                name: param_name,
                param_type,
            });
        }

        Ok(ContractEvent { name, parameters })
    }

    // Parse a type string into a ContractValueType
    fn parse_type(type_str: &str) -> IcnResult<ContractValueType> {
        match type_str {
            "int" => Ok(ContractValueType::Integer),
            "float" => Ok(ContractValueType::Float),
            "bool" => Ok(ContractValueType::Boolean),
            "string" => Ok(ContractValueType::String),
            "address" => Ok(ContractValueType::Address),
            _ if type_str.starts_with("list<") => {
                let inner_type = &type_str[5..type_str.len()-1];
                Ok(ContractValueType::List(Box::new(Self::parse_type(inner_type)?)))
            }
            _ if type_str.starts_with("map<") => {
                let inner_types: Vec<&str> = type_str[4..type_str.len()-1].split(',').collect();
                if inner_types.len() != 2 {
                    return Err(IcnError::SmartContract("Invalid map type format".into()));
                }
                Ok(ContractValueType::Map(
                    Box::new(Self::parse_type(inner_types[0].trim())?),
                    Box::new(Self::parse_type(inner_types[1].trim())?)))
            }
            _ => Err(IcnError::SmartContract(format!("Unknown type: {}", type_str))),
        }
    }
}

/// Struct to represent the execution context of a smart contract
pub struct ContractContext {
    pub balances: HashMap<String, HashMap<CurrencyType, f64>>,
    pub storage: HashMap<String, Value>,
    pub block_height: u64,
    pub timestamp: u64,
    pub caller: String,
}

/// Smart contract executor
pub struct SmartContractExecutor {
    vm: CoopVM,
}

impl SmartContractExecutor {
    pub fn new() -> Self {
        SmartContractExecutor {
            vm: CoopVM::new(Vec::new()),
        }
    }

    /// Execute a compiled smart contract
    pub fn execute(&mut self, contract: &CompiledContract, context: &mut ContractContext, function: &str, args: Vec<Value>) -> IcnResult<Option<Value>> {
        self.vm.load_program(contract.bytecode.clone());
        self.vm.set_context(context);

        // Find the function in the ABI
        let function_abi = contract.abi.functions.iter()
            .find(|f| f.name == function)
            .ok_or_else(|| IcnError::SmartContract(format!("Function {} not found", function)))?;

        // Check argument count
        if args.len() != function_abi.inputs.len() {
            return Err(IcnError::SmartContract("Incorrect number of arguments".into()));
        }

        // Push arguments onto the stack
        for arg in args {
            self.vm.push(arg);
        }

        // Call the function
        self.vm.call(function)?;

        // Run the VM
        self.vm.run()?;

        // Return the top value from the stack, if any
        Ok(self.vm.pop())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_contract() {
        let contract_source = r#"
        contract_type: AssetTransfer

        function transfer(from: address, to: address, amount: int) {
            if balanceOf(from) >= amount {
                balanceOf[from] = balanceOf[from] - amount;
                balanceOf[to] = balanceOf[to] + amount;
                emit Transfer(from, to, amount);
            }
        }

        event Transfer(from: address, to: address, amount: int)
        "#;

        let compiled_contract = NaturalLanguageCompiler::compile(contract_source).unwrap();

        assert_eq!(compiled_contract.contract_type, SmartContractType::AssetTransfer);
        assert!(!compiled_contract.bytecode.is_empty());
        assert_eq!(compiled_contract.abi.functions.len(), 1);
        assert_eq!(compiled_contract.abi.events.len(), 1);

        let transfer_function = &compiled_contract.abi.functions[0];
        assert_eq!(transfer_function.name, "transfer");
        assert_eq!(transfer_function.inputs.len(), 3);

        let transfer_event = &compiled_contract.abi.events[0];
        assert_eq!(transfer_event.name, "Transfer");
        assert_eq!(transfer_event.parameters.len(), 3);
    }

    #[test]
    fn test_execute_simple_contract() {
        let contract_source = r#"
        contract_type: AssetTransfer

        function transfer(from: address, to: address, amount: int) {
            if balanceOf(from) >= amount {
                balanceOf[from] = balanceOf[from] - amount;
                balanceOf[to] = balanceOf[to] + amount;
                emit Transfer(from, to, amount);
            }
        }

        event Transfer(from: address, to: address, amount: int)
        "#;

        let compiled_contract = NaturalLanguageCompiler::compile(contract_source).unwrap();
        let mut executor = SmartContractExecutor::new();

        let mut context = ContractContext {
            balances: HashMap::new(),
            storage: HashMap::new(),
            block_height: 1,
            timestamp: 1623456789,
            caller: "system".to_string(),
        };

        // Initialize balances
        let mut from_balance = HashMap::new();
        from_balance.insert(CurrencyType::BasicNeeds, 100.0);
        context.balances.insert("Alice".to_string(), from_balance);

        let mut to_balance = HashMap::new();
        to_balance.insert(CurrencyType::BasicNeeds, 50.0);
        context.balances.insert("Bob".to_string(), to_balance);

        // Execute transfer
        let result = executor.execute(
            &compiled_contract,
            &mut context,
            "transfer",
            vec![
                Value::String("Alice".to_string()),
                Value::String("Bob".to_string()),
                Value::Int(30),
            ],
        );

        assert!(result.is_ok());

        // Check balances after transfer
        assert_eq!(context.balances["Alice"][&CurrencyType::BasicNeeds], 70.0);
        assert_eq!(context.balances["Bob"][&CurrencyType::BasicNeeds], 80.0);
    }
}

// Helper trait for CoopVM to interact with ContractContext
trait VMContext {
    fn get_balance(&self, address: &str, currency: &CurrencyType) -> f64;
    fn set_balance(&mut self, address: &str, currency: &CurrencyType, amount: f64);
    fn get_storage(&self, key: &str) -> Option<&Value>;
    fn set_storage(&mut self, key: String, value: Value);
    fn emit_event(&mut self, name: &str, params: Vec<Value>);
}

impl VMContext for ContractContext {
    fn get_balance(&self, address: &str, currency: &CurrencyType) -> f64 {
        self.balances.get(address).and_then(|balances| balances.get(currency)).cloned().unwrap_or(0.0)
    }

    fn set_balance(&mut self, address: &str, currency: &CurrencyType, amount: f64) {
        self.balances.entry(address.to_string()).or_insert_with(HashMap::new).insert(currency.clone(), amount);
    }

    fn get_storage(&self, key: &str) -> Option<&Value> {
        self.storage.get(key)
    }

    fn set_storage(&mut self, key: String, value: Value) {
        self.storage.insert(key, value);
    }

    fn emit_event(&mut self, name: &str, params: Vec<Value>) {
        println!("Event emitted: {} {:?}", name, params);
    }
}

// Extend CoopVM to work with ContractContext
impl CoopVM {
    pub fn set_context(&mut self, _context: &ContractContext) {
        // Implementation depends on how CoopVM is designed to interact with external state
    }

    pub fn push(&mut self, _value: Value) {
        // Implementation to push a value onto the VM's stack
    }

    pub fn pop(&mut self) -> Option<Value> {
        // Implementation to pop a value from the VM's stack
        None
    }

    pub fn call(&mut self, _function: &str) -> IcnResult<()> {
        // Implementation to call a function in the VM
        Ok(())
    }
}
