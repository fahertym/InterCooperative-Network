use crate::{Statement, Expression, Value};
use icn_vm::{Opcode, Value as VMValue};

pub struct Compiler {
    opcodes: Vec<Opcode>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler { opcodes: Vec::new() }
    }

    pub fn compile(&mut self, statements: Vec<Statement>) -> Vec<Opcode> {
        for statement in statements {
            self.compile_statement(statement);
        }
        self.opcodes.clone()
    }

    fn compile_statement(&mut self, statement: Statement) {
        match statement {
            Statement::VariableDeclaration(name, expr) => {
                self.compile_expression(expr);
                self.opcodes.push(Opcode::Store(name));
            },
            Statement::FunctionCall(name, args) => {
                for arg in args {
                    self.compile_expression(arg);
                }
                self.opcodes.push(Opcode::MorphemicCall(name, args.len()));
            },
            Statement::IfStatement(condition, true_branch, false_branch) => {
                self.compile_expression(condition);
                let jump_false = self.opcodes.len();
                self.opcodes.push(Opcode::JumpIfFalse(0)); // Placeholder
                self.compile_statements(true_branch);
                if let Some(false_branch) = false_branch {
                    let jump_end = self.opcodes.len();
                    self.opcodes.push(Opcode::Jump(0)); // Placeholder
                    let false_start = self.opcodes.len();
                    self.compile_statements(false_branch);
                    let end = self.opcodes.len();
                    self.opcodes[jump_false] = Opcode::JumpIfFalse(false_start);
                    self.opcodes[jump_end] = Opcode::Jump(end);
                } else {
                    let end = self.opcodes.len();
                    self.opcodes[jump_false] = Opcode::JumpIfFalse(end);
                }
            },
            Statement::WhileLoop(condition, body) => {
                let loop_start = self.opcodes.len();
                self.compile_expression(condition);
                let jump_false = self.opcodes.len();
                self.opcodes.push(Opcode::JumpIfFalse(0)); // Placeholder
                self.compile_statements(body);
                self.opcodes.push(Opcode::Jump(loop_start));
                let end = self.opcodes.len();
                self.opcodes[jump_false] = Opcode::JumpIfFalse(end);
            },
        }
    }

    fn compile_statements(&mut self, statements: Vec<Statement>) {
        for statement in statements {
            self.compile_statement(statement);
        }
    }

    fn compile_expression(&mut self, expr: Expression) {
        match expr {
            Expression::Literal(value) => {
                self.opcodes.push(Opcode::Push(self.convert_value(value)));
            },
            Expression::Variable(name) => {
                self.opcodes.push(Opcode::Load(name));
            },
            Expression::FunctionCall(name, args) => {
                for arg in args {
                    self.compile_expression(arg);
                }
                self.opcodes.push(Opcode::MorphemicCall(name, args.len()));
            },
        }
    }

    fn convert_value(&self, value: Value) -> VMValue {
        match value {
            Value::Int(i) => VMValue::Int(i),
            Value::Float(f) => VMValue::Float(f),
            Value::Bool(b) => VMValue::Bool(b),
            Value::String(s) => VMValue::String(s),
            Value::Address(a) => VMValue::Address(a),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile;

    #[test]
    fn test_compile_program() {
        let source = r#"
            let x = 5
            while x < 10 {
                net-node-connect("node1", "node2")
                x = x + 1
            }
            if x == 10 {
                econ-currency-mint(100, "BasicNeeds")
            } else {
                gov-proposal-submit("Increase node count")
            }
        "#;
        let ast = compile(source).unwrap();
        let mut compiler = Compiler::new();
        let opcodes = compiler.compile(ast);
        
        // Assert the number of opcodes and some key instructions
        assert!(opcodes.len() > 10);
        assert!(opcodes.contains(&Opcode::Push(VMValue::Int(5))));
        assert!(opcodes.contains(&Opcode::MorphemicCall("net-node-connect".to_string(), 2)));
        assert!(opcodes.contains(&Opcode::MorphemicCall("econ-currency-mint".to_string(), 2)));
        assert!(opcodes.contains(&Opcode::MorphemicCall("gov-proposal-submit".to_string(), 1)));
    }
}