use pest::Parser;
use pest_derive::Parser;
use icn_common::{IcnResult, IcnError};
use icn_vm::{CoopVM, Opcode, Value};
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "contract.pest"]
struct ContractParser;

pub struct NaturalLanguageCompiler;

impl NaturalLanguageCompiler {
    pub fn compile(input: &str) -> IcnResult<Vec<Opcode>> {
        let pairs = ContractParser::parse(Rule::contract, input)
            .map_err(|e| IcnError::SmartContract(format!("Parsing error: {}", e)))?;

        let mut opcodes = Vec::new();

        for pair in pairs {
            match pair.as_rule() {
                Rule::statement => {
                    opcodes.extend(Self::compile_statement(pair)?);
                }
                _ => {}
            }
        }

        Ok(opcodes)
    }

    fn compile_statement(pair: pest::iterators::Pair<Rule>) -> IcnResult<Vec<Opcode>> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::assignment => Self::compile_assignment(inner),
            Rule::if_statement => Self::compile_if_statement(inner),
            Rule::function_call => Self::compile_function_call(inner),
            _ => Err(IcnError::SmartContract("Unknown statement type".into())),
        }
    }

    fn compile_assignment(pair: pest::iterators::Pair<Rule>) -> IcnResult<Vec<Opcode>> {
        let mut inner = pair.into_inner();
        let variable = inner.next().unwrap().as_str();
        let value = inner.next().unwrap();

        let mut opcodes = Self::compile_expression(value)?;
        opcodes.push(Opcode::Store(variable.to_string()));
        Ok(opcodes)
    }

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
}

pub struct ContractContext {
    pub balances: HashMap<String, f64>,
    pub votes: HashMap<String, bool>,
    pub reputation: HashMap<String, i32>,
}

impl ContractContext {
    pub fn new() -> Self {
        ContractContext {
            balances: HashMap::new(),
            votes: HashMap::new(),
            reputation: HashMap::new(),
        }
    }
}

pub struct SmartContractExecutor {
    vm: CoopVM,
}

impl SmartContractExecutor {
    pub fn new() -> Self {
        SmartContractExecutor {
            vm: CoopVM::new(Vec::new()),
        }
    }

    pub fn execute(&mut self, contract: &str, context: &mut ContractContext) -> IcnResult<Option<Value>> {
        let opcodes = NaturalLanguageCompiler::compile(contract)?;
        self.vm.load_program(opcodes);
        self.vm.set_context(context);
        self.vm.run()?;
        Ok(self.vm.get_stack().last().cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_language_compiler() {
        let contract = r#"
        let x = 5
        let y = 10
        if x < y {
            let z = x + y
        } else {
            let z = x - y
        }
        vote("proposal1", true)
        "#;

        let opcodes = NaturalLanguageCompiler::compile(contract).unwrap();
        assert!(!opcodes.is_empty());
    }

    #[test]
    fn test_smart_contract_executor() {
        let mut executor = SmartContractExecutor::new();
        let mut context = ContractContext::new();
        context.balances.insert("Alice".to_string(), 100.0);
        context.balances.insert("Bob".to_string(), 50.0);

        let contract = r#"
        let x = 5
        let y = 10
        let z = x + y
        transfer("Alice", "Bob", z)
        get_balance("Bob")
        "#;

        let result = executor.execute(contract, &mut context).unwrap();
        assert_eq!(result, Some(Value::Float(65.0)));
    }
}