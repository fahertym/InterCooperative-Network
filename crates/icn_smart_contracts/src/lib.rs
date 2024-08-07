// File: crates/icn_smart_contracts/src/lib.rs

use pest::Parser;
use pest_derive::Parser;
use icn_common::{IcnResult, IcnError};
use icn_vm::{CoopVM, Opcode, Value};
use std::collections::HashMap;
use std::fmt;

#[derive(Parser)]
#[grammar = "contract.pest"]
struct ContractParser;

#[derive(Debug, Clone)]
pub enum SmartContractType {
    AssetTransfer,
    VotingSystem,
    ReputationManagement,
    ResourceAllocation,
    CustomLogic,
}

#[derive(Debug)]
pub struct CompiledContract {
    contract_type: SmartContractType,
    bytecode: Vec<Opcode>,
    abi: ContractABI,
}

#[derive(Debug)]
pub struct ContractABI {
    functions: Vec<ContractFunction>,
    events: Vec<ContractEvent>,
}

#[derive(Debug)]
pub struct ContractFunction {
    name: String,
    inputs: Vec<ContractParameter>,
    outputs: Vec<ContractParameter>,
}

#[derive(Debug)]
pub struct ContractEvent {
    name: String,
    parameters: Vec<ContractParameter>,
}

#[derive(Debug)]
pub struct ContractParameter {
    name: String,
    param_type: ContractValueType,
}

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

pub struct NaturalLanguageCompiler;

impl NaturalLanguageCompiler {
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

    fn compile_function(pair: pest::iterators::Pair<Rule>) -> IcnResult<(Vec<Opcode>, ContractFunction)> {
        let mut bytecode = Vec::new();
        let mut function = ContractFunction {
            name: String::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        };

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::identifier => function.name = inner_pair.as_str().to_string(),
                Rule::parameter_list => {
                    for param in inner_pair.into_inner() {
                        let param_pair = param.into_inner().next().unwrap();
                        let param_name = param_pair.as_str().to_string();
                        let param_type = Self::parse_type(param_pair.into_inner().next().unwrap().as_str())?;
                        function.inputs.push(ContractParameter { name: param_name, param_type });
                    }
                }
                Rule::statement => bytecode.extend(Self::compile_statement(inner_pair)?),
                _ => {}
            }
        }

        Ok((bytecode, function))
    }

    fn compile_statement(pair: pest::iterators::Pair<Rule>) -> IcnResult<Vec<Opcode>> {
        let mut bytecode = Vec::new();

        match pair.as_rule() {
            Rule::assignment => {
                let mut inner = pair.into_inner();
                let var_name = inner.next().unwrap().as_str().to_string();
                let expr = inner.next().unwrap();
                bytecode.extend(Self::compile_expression(expr)?);
                bytecode.push(Opcode::Store(var_name));
            }
            Rule::if_statement => {
                let mut inner = pair.into_inner();
                let condition = inner.next().unwrap();
                let if_block = inner.next().unwrap();
                let else_block = inner.next();

                bytecode.extend(Self::compile_expression(condition)?);
                let jump_to_else = bytecode.len();
                bytecode.push(Opcode::JumpIf(0)); // Placeholder, will be updated later

                bytecode.extend(Self::compile_statement(if_block)?);
                let jump_to_end = bytecode.len();
                bytecode.push(Opcode::Jump(0)); // Placeholder, will be updated later

                let else_start = bytecode.len();
                if let Some(else_block) = else_block {
                    bytecode.extend(Self::compile_statement(else_block)?);
                }
                let end = bytecode.len();

                bytecode[jump_to_else] = Opcode::JumpIf(else_start);
                bytecode[jump_to_end] = Opcode::Jump(end);
            }
            Rule::while_loop => {
                let mut inner = pair.into_inner();
                let condition = inner.next().unwrap();
                let body = inner.next().unwrap();

                let loop_start = bytecode.len();
                bytecode.extend(Self::compile_expression(condition)?);
                let jump_to_end = bytecode.len();
                bytecode.push(Opcode::JumpIf(0)); // Placeholder, will be updated later

                bytecode.extend(Self::compile_statement(body)?);
                bytecode.push(Opcode::Jump(loop_start));

                let end = bytecode.len();
                bytecode[jump_to_end] = Opcode::JumpIf(end);
            }
            Rule::function_call => {
                bytecode.extend(Self::compile_function_call(pair)?);
            }
            _ => return Err(IcnError::SmartContract("Unsupported statement type".into())),
        }

        Ok(bytecode)
    }

    fn compile_expression(pair: pest::iterators::Pair<Rule>) -> IcnResult<Vec<Opcode>> {
        let mut bytecode = Vec::new();

        match pair.as_rule() {
            Rule::literal => {
                let value = Self::parse_literal(pair)?;
                bytecode.push(Opcode::Push(value));
            }
            Rule::identifier => {
                let var_name = pair.as_str().to_string();
                bytecode.push(Opcode::Load(var_name));
            }
            Rule::binary_operation => {
                let mut inner = pair.into_inner();
                let left = inner.next().unwrap();
                let op = inner.next().unwrap();
                let right = inner.next().unwrap();

                bytecode.extend(Self::compile_expression(left)?);
                bytecode.extend(Self::compile_expression(right)?);

                match op.as_str() {
                    "+" => bytecode.push(Opcode::Add),
                    "-" => bytecode.push(Opcode::Sub),
                    "*" => bytecode.push(Opcode::Mul),
                    "/" => bytecode.push(Opcode::Div),
                    "==" => bytecode.push(Opcode::Eq),
                    "!=" => bytecode.push(Opcode::Neq),
                    ">" => bytecode.push(Opcode::Gt),
                    "<" => bytecode.push(Opcode::Lt),
                    ">=" => bytecode.push(Opcode::Gte),
                    "<=" => bytecode.push(Opcode::Lte),
                    "&&" => bytecode.push(Opcode::And),
                    "||" => bytecode.push(Opcode::Or),
                    _ => return Err(IcnError::SmartContract(format!("Unsupported operator: {}", op.as_str()))),
                }
            }
            Rule::function_call => {
                bytecode.extend(Self::compile_function_call(pair)?);
            }
            _ => return Err(IcnError::SmartContract("Unsupported expression type".into())),
        }

        Ok(bytecode)
    }

    fn compile_function_call(pair: pest::iterators::Pair<Rule>) -> IcnResult<Vec<Opcode>> {
        let mut bytecode = Vec::new();
        let mut inner = pair.into_inner();
        let func_name = inner.next().unwrap().as_str().to_string();

        for arg in inner {
            bytecode.extend(Self::compile_expression(arg)?);
        }

        bytecode.push(Opcode::Call(func_name));
        Ok(bytecode)
    }

    fn compile_event(pair: pest::iterators::Pair<Rule>) -> IcnResult<ContractEvent> {
        let mut event = ContractEvent {
            name: String::new(),
            parameters: Vec::new(),
        };

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::identifier => event.name = inner_pair.as_str().to_string(),
                Rule::parameter_list => {
                    for param in inner_pair.into_inner() {
                        let param_pair = param.into_inner().next().unwrap();
                        let param_name = param_pair.as_str().to_string();
                        let param_type = Self::parse_type(param_pair.into_inner().next().unwrap().as_str())?;
                        event.parameters.push(ContractParameter { name: param_name, param_type });
                    }
                }
                _ => {}
            }
        }

        Ok(event)
    }

    fn parse_literal(pair: pest::iterators::Pair<Rule>) -> IcnResult<Value> {
        match pair.as_rule() {
            Rule::integer => Ok(Value::Int(pair.as_str().parse().map_err(|e| IcnError::SmartContract(format!("Invalid integer: {}", e)))?)),
            Rule::float => Ok(Value::Float(pair.as_str().parse().map_err(|e| IcnError::SmartContract(format!("Invalid float: {}", e)))?)),
            Rule::boolean => Ok(Value::Bool(pair.as_str().parse().map_err(|e| IcnError::SmartContract(format!("Invalid boolean: {}", e)))?)),
            Rule::string => Ok(Value::String(pair.into_inner().next().unwrap().as_str().to_string())),
            _ => Err(IcnError::SmartContract("Unsupported literal type".into())),
        }
    }

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

pub struct SmartContractExecutor {
    vm: CoopVM,
    contracts: HashMap<String, CompiledContract>,
}

impl SmartContractExecutor {
    pub fn new() -> Self {
        SmartContractExecutor {
            vm: CoopVM::new(),
            contracts: HashMap::new(),
        }
    }

    pub fn deploy_contract(&mut self, contract_id: String, contract: CompiledContract) -> IcnResult<()> {
        if self.contracts.contains_key(&contract_id) {
            return Err(IcnError::SmartContract(format!("Contract with ID {} already exists", contract_id)));
        }
        self.contracts.insert(contract_id, contract);
        Ok(())
    }

    pub fn execute_contract(&mut self, contract_id: &str, function: &str, args: Vec<Value>) -> IcnResult<Option<Value>> {
        let contract = self.contracts.get(contract_id)
            .ok_or_else(|| IcnError::SmartContract(format!("Contract with ID {} not found", contract_id)))?;

        let function_abi = contract.abi.functions.iter()
            .find(|f| f.name == function)
            .ok_or_else(|| IcnError::SmartContract(format!("Function {} not found in contract {}", function, contract_id)))?;

        if args.len() != function_abi.inputs.len() {
            return Err(IcnError::SmartContract(format!("Invalid number of arguments for function {}", function)));
        }

        self.vm.load_program(contract.bytecode.clone());

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

    pub fn get_contract_state(&self, contract_id: &str) -> IcnResult<&HashMap<String, Value>> {
        self.contracts.get(contract_id)
            .map(|contract| self.vm.get_memory())
            .ok_or_else(|| IcnError::SmartContract(format!("Contract with ID {} not found", contract_id)))
    }

    pub fn update_contract_state(&mut self, contract_id: &str, key: String, value: Value) -> IcnResult<()> {
        if !self.contracts.contains_key(contract_id) {
            return Err(IcnError::SmartContract(format!("Contract with ID {} not found", contract_id)));
        }

        self.vm.store(&key, value);
        Ok(())
    }

    pub fn get_contract(&self, contract_id: &str) -> IcnResult<&CompiledContract> {
        self.contracts.get(contract_id)
            .ok_or_else(|| IcnError::SmartContract(format!("Contract with ID {} not found", contract_id)))
    }

    pub fn list_contracts(&self) -> Vec<String> {
        self.contracts.keys().cloned().collect()
    }

    pub fn remove_contract(&mut self, contract_id: &str) -> IcnResult<()> {
        self.contracts.remove(contract_id)
            .ok_or_else(|| IcnError::SmartContract(format!("Contract with ID {} not found", contract_id)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_language_compiler() {
        let input = r#"
            contract AssetTransfer

            function transfer(from: address, to: address, amount: int) {
                if balance[from] >= amount {
                    balance[from] = balance[from] - amount
                    balance[to] = balance[to] + amount
                    emit Transfer(from, to, amount)
                }
            }

            event Transfer(from: address, to: address, amount: int)
        "#;

        let compiled_contract = NaturalLanguageCompiler::compile(input).unwrap();
        assert_eq!(compiled_contract.contract_type, SmartContractType::AssetTransfer);
        assert!(!compiled_contract.bytecode.is_empty());
        assert_eq!(compiled_contract.abi.functions.len(), 1);
        assert_eq!(compiled_contract.abi.events.len(), 1);
    }

    #[test]
    fn test_smart_contract_executor() {
        let mut executor = SmartContractExecutor::new();

        let contract = CompiledContract {
            contract_type: SmartContractType::AssetTransfer,
            bytecode: vec![
                Opcode::Push(Value::Int(100)),
                Opcode::Store("balance".to_string()),
                Opcode::Load("balance".to_string()),
                Opcode::Push(Value::Int(50)),
                Opcode::Sub,
                Opcode::Store("balance".to_string()),
                Opcode::Load("balance".to_string()),
            ],
            abi: ContractABI {
                functions: vec![
                    ContractFunction {
                        name: "transfer".to_string(),
                        inputs: vec![],
                        outputs: vec![],
                    }
                ],
                events: vec![],
            },
        };

        executor.deploy_contract("test_contract".to_string(), contract).unwrap();

        let result = executor.execute_contract("test_contract", "transfer", vec![]).unwrap();
        assert_eq!(result, Some(Value::Int(50)));

        let state = executor.get_contract_state("test_contract").unwrap();
        assert_eq!(state.get("balance"), Some(&Value::Int(50)));
    }
}