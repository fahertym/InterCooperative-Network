use crate::vm::opcode::{Opcode, Value};
use std::error::Error;

// Define the tokens that the lexer will generate from source code
#[derive(Debug, PartialEq, Clone)]
enum Token {
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    True,
    False,
    If,
    Else,
    While,
    Function,
    Return,
    Vote,
    AllocateResource,
    UpdateReputation,
    CreateProposal,
    GetProposalStatus,
    Emit,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Comma,
    Equals,
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    DoubleEquals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanEquals,
    LessThanEquals,
    And,
    Or,
    Not,
}

// Lexer for converting source code into tokens
struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    // Create a new lexer with the given input string
    fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    // Get the next token from the input
    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return None;
        }

        match self.input[self.position] {
            '(' => {
                self.position += 1;
                Some(Token::LParen)
            }
            ')' => {
                self.position += 1;
                Some(Token::RParen)
            }
            '{' => {
                self.position += 1;
                Some(Token::LBrace)
            }
            '}' => {
                self.position += 1;
                Some(Token::RBrace)
            }
            ';' => {
                self.position += 1;
                Some(Token::Semicolon)
            }
            ',' => {
                self.position += 1;
                Some(Token::Comma)
            }
            '+' => {
                self.position += 1;
                Some(Token::Plus)
            }
            '-' => {
                self.position += 1;
                Some(Token::Minus)
            }
            '*' => {
                self.position += 1;
                Some(Token::Multiply)
            }
            '/' => {
                self.position += 1;
                Some(Token::Divide)
            }
            '%' => {
                self.position += 1;
                Some(Token::Modulo)
            }
            '=' => {
                if self.peek_next() == Some('=') {
                    self.position += 2;
                    Some(Token::DoubleEquals)
                } else {
                    self.position += 1;
                    Some(Token::Equals)
                }
            }
            '!' => {
                if self.peek_next() == Some('=') {
                    self.position += 2;
                    Some(Token::NotEquals)
                } else {
                    self.position += 1;
                    Some(Token::Not)
                }
            }
            '>' => {
                if self.peek_next() == Some('=') {
                    self.position += 2;
                    Some(Token::GreaterThanEquals)
                } else {
                    self.position += 1;
                    Some(Token::GreaterThan)
                }
            }
            '<' => {
                if self.peek_next() == Some('=') {
                    self.position += 2;
                    Some(Token::LessThanEquals)
                } else {
                    self.position += 1;
                    Some(Token::LessThan)
                }
            }
            '&' => {
                if self.peek_next() == Some('&') {
                    self.position += 2;
                    Some(Token::And)
                } else {
                    None // Invalid token
                }
            }
            '|' => {
                if self.peek_next() == Some('|') {
                    self.position += 2;
                    Some(Token::Or)
                } else {
                    None // Invalid token
                }
            }
            '"' => Some(self.read_string()),
            c if c.is_alphabetic() => Some(self.read_identifier()),
            c if c.is_digit(10) => Some(self.read_number()),
            _ => None, // Invalid token
        }
    }

    // Skip whitespace characters in the input
    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    // Peek at the next character without advancing the position
    fn peek_next(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }

    // Read a string token from the input
    fn read_string(&mut self) -> Token {
        self.position += 1; // Skip opening quote
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position] != '"' {
            self.position += 1;
        }
        let value: String = self.input[start..self.position].iter().collect();
        self.position += 1; // Skip closing quote
        Token::String(value)
    }

    // Read an identifier token from the input
    fn read_identifier(&mut self) -> Token {
        let start = self.position;
        while self.position < self.input.len() && (self.input[self.position].is_alphanumeric() || self.input[self.position] == '_') {
            self.position += 1;
        }
        let value: String = self.input[start..self.position].iter().collect();
        match value.as_str() {
            "true" => Token::True,
            "false" => Token::False,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "function" => Token::Function,
            "return" => Token::Return,
            "vote" => Token::Vote,
            "allocate_resource" => Token::AllocateResource,
            "update_reputation" => Token::UpdateReputation,
            "create_proposal" => Token::CreateProposal,
            "get_proposal_status" => Token::GetProposalStatus,
            "emit" => Token::Emit,
            _ => Token::Identifier(value),
        }
    }

    // Read a number token from the input
    fn read_number(&mut self) -> Token {
        let start = self.position;
        let mut is_float = false;
        while self.position < self.input.len() && (self.input[self.position].is_digit(10) || self.input[self.position] == '.') {
            if self.input[self.position] == '.' {
                is_float = true;
            }
            self.position += 1;
        }
        let value: String = self.input[start..self.position].iter().collect();
        if is_float {
            Token::Float(value.parse().unwrap())
        } else {
            Token::Integer(value.parse().unwrap())
        }
    }
}

// Parser for converting tokens into opcodes
struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    // Create a new parser with the given tokens
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    // Parse the tokens into a vector of opcodes
    fn parse(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        let mut opcodes = Vec::new();
        while self.position < self.tokens.len() {
            opcodes.append(&mut self.parse_statement()?);
        }
        Ok(opcodes)
    }

    // Parse a single statement into opcodes
    fn parse_statement(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        match self.current_token() {
            Some(Token::If) => self.parse_if_statement(),
            Some(Token::While) => self.parse_while_statement(),
            Some(Token::Function) => self.parse_function_definition(),
            Some(Token::Return) => self.parse_return_statement(),
            Some(Token::Identifier(_)) => self.parse_assignment_or_function_call(),
            Some(Token::Vote) => self.parse_vote_statement(),
            Some(Token::AllocateResource) => self.parse_allocate_resource_statement(),
            Some(Token::UpdateReputation) => self.parse_update_reputation_statement(),
            Some(Token::CreateProposal) => self.parse_create_proposal_statement(),
            Some(Token::GetProposalStatus) => self.parse_get_proposal_status_statement(),
            Some(Token::Emit) => self.parse_emit_statement(),
            _ => Err("Unexpected token in statement".into()),
        }
    }

    // Parse an if statement into opcodes
    fn parse_if_statement(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        // Implementation for parsing if statements
        Err("If statement parsing not implemented yet".into())
    }

    // Parse a while loop into opcodes
    fn parse_while_statement(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        // Implementation for parsing while loops
        Err("While statement parsing not implemented yet".into())
    }

    // Parse a function definition into opcodes
    fn parse_function_definition(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        // Implementation for parsing function definitions
        Err("Function definition parsing not implemented yet".into())
    }

    // Parse a return statement into opcodes
    fn parse_return_statement(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        self.consume_token(Token::Return)?;
        let mut opcodes = self.parse_expression()?;
        opcodes.push(Opcode::Return);
        self.consume_token(Token::Semicolon)?;
        Ok(opcodes)
    }

    // Parse an assignment or function call into opcodes
    fn parse_assignment_or_function_call(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        let identifier = self.consume_identifier()?;
        match self.current_token() {
            Some(Token::Equals) => self.parse_assignment(identifier),
            Some(Token::LParen) => self.parse_function_call(identifier),
            _ => Err("Expected '=' or '(' after identifier".into()),
        }
    }

    // Parse an assignment statement into opcodes
    fn parse_assignment(&mut self, identifier: String) -> Result<Vec<Opcode>, Box<dyn Error>> {
        self.consume_token(Token::Equals)?;
        let mut opcodes = self.parse_expression()?;
        opcodes.push(Opcode::Store(identifier));
        self.consume_token(Token::Semicolon)?;
        Ok(opcodes)
    }

    // Parse a function call into opcodes
    fn parse_function_call(&mut self, identifier: String) -> Result<Vec<Opcode>, Box<dyn Error>> {
        self.consume_token(Token::LParen)?;
        let mut opcodes = Vec::new();
        while !matches!(self.current_token(), Some(Token::RParen)) {
            opcodes.append(&mut self.parse_expression()?);
            if matches!(self.current_token(), Some(Token::Comma)) {
                self.consume_token(Token::Comma)?;
            }
        }
        self.consume_token(Token::RParen)?;
        opcodes.push(Opcode::Call(identifier));
        self.consume_token(Token::Semicolon)?;
        Ok(opcodes)
    }

    // Parse a vote statement into opcodes
    fn parse_vote_statement(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        self.consume_token(Token::Vote)?;
        self.consume_token(Token::LParen)?;
        let proposal_id = self.consume_string()?;
        self.consume_token(Token::Comma)?;
        let mut opcodes = self.parse_expression()?; // This should push a boolean onto the stack
        self.consume_token(Token::RParen)?;
        opcodes.push(Opcode::Vote(proposal_id));
        self.consume_token(Token::Semicolon)?;
        Ok(opcodes)
    }

    // Parse an allocate resource statement into opcodes
    fn parse_allocate_resource_statement(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        self.consume_token(Token::AllocateResource)?;
        self.consume_token(Token::LParen)?;
        let resource_id = self.consume_string()?;
        self.consume_token(Token::Comma)?;
        let mut opcodes = self.parse_expression()?; // This should push an integer onto the stack
        self.consume_token(Token::RParen)?;
        opcodes.push(Opcode::AllocateResource(resource_id));
        self.consume_token(Token::Semicolon)?;
        Ok(opcodes)
    }

    // Parse an update reputation statement into opcodes
    fn parse_update_reputation_statement(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        self.consume_token(Token::UpdateReputation)?;
        self.consume_token(Token::LParen)?;
        let address = self.consume_string()?;
        self.consume_token(Token::Comma)?;
        let mut opcodes = self.parse_expression()?; // This should push an integer onto the stack
        self.consume_token(Token::RParen)?;
        opcodes.push(Opcode::UpdateReputation(address));
        self.consume_token(Token::Semicolon)?;
        Ok(opcodes)
    }

    // Parse a create proposal statement into opcodes
    fn parse_create_proposal_statement(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        self.consume_token(Token::CreateProposal)?;
        self.consume_token(Token::LParen)?;
        let mut opcodes = self.parse_expression()?; // This should push a string onto the stack
        self.consume_token(Token::RParen)?;
        opcodes.push(Opcode::CreateProposal);
        self.consume_token(Token::Semicolon)?;
        Ok(opcodes)
    }

    // Parse a get proposal status statement into opcodes
    fn parse_get_proposal_status_statement(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        self.consume_token(Token::GetProposalStatus)?;
        self.consume_token(Token::LParen)?;
        let mut opcodes = self.parse_expression()?; // This should push a string onto the stack
        self.consume_token(Token::RParen)?;
        opcodes.push(Opcode::GetProposalStatus);
        self.consume_token(Token::Semicolon)?;
        Ok(opcodes)
    }

    // Parse an emit statement into opcodes
    fn parse_emit_statement(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        self.consume_token(Token::Emit)?;
        self.consume_token(Token::LParen)?;
        let event_name = self.consume_string()?;
        self.consume_token(Token::Comma)?;
        let mut opcodes = self.parse_expression()?; // This should push the event data onto the stack
        self.consume_token(Token::RParen)?;
        opcodes.push(Opcode::Emit(event_name));
        self.consume_token(Token::Semicolon)?;
        Ok(opcodes)
    }

    // Parse an expression into opcodes
    fn parse_expression(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        let mut opcodes = self.parse_term()?;

        while let Some(token) = self.current_token() {
            match token {
                Token::Plus => {
                    self.position += 1;
                    opcodes.append(&mut self.parse_term()?);
                    opcodes.push(Opcode::Add);
                }
                Token::Minus => {
                    self.position += 1;
                    opcodes.append(&mut self.parse_term()?);
                    opcodes.push(Opcode::Sub);
                }
                _ => break,
            }
        }

        Ok(opcodes)
    }

    fn parse_term(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        let mut opcodes = self.parse_factor()?;

        while let Some(token) = self.current_token() {
            match token {
                Token::Multiply => {
                    self.position += 1;
                    opcodes.append(&mut self.parse_factor()?);
                    opcodes.push(Opcode::Mul);
                }
                Token::Divide => {
                    self.position += 1;
                    opcodes.append(&mut self.parse_factor()?);
                    opcodes.push(Opcode::Div);
                }
                _ => break,
            }
        }

        Ok(opcodes)
    }

    fn parse_factor(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        let token = self.current_token().cloned();
        match token {
            Some(Token::Integer(value)) => {
                self.position += 1;
                Ok(vec![Opcode::Push(Value::Int(value))])
            }
            Some(Token::Float(value)) => {
                self.position += 1;
                Ok(vec![Opcode::Push(Value::Float(value))])
            }
            Some(Token::String(value)) => {
                self.position += 1;
                Ok(vec![Opcode::Push(Value::String(value))])
            }
            Some(Token::True) => {
                self.position += 1;
                Ok(vec![Opcode::Push(Value::Bool(true))])
            }
            Some(Token::False) => {
                self.position += 1;
                Ok(vec![Opcode::Push(Value::Bool(false))])
            }
            Some(Token::Identifier(name)) => {
                self.position += 1;
                Ok(vec![Opcode::Load(name)])
            }
            Some(Token::LParen) => {
                self.position += 1;
                let expr = self.parse_expression()?;
                self.consume_token(Token::RParen)?;
                Ok(expr)
            }
            _ => Err("Unexpected token in expression".into()),
        }
    }

    // Consume the next token if it matches the expected token
    fn consume_token(&mut self, expected: Token) -> Result<(), Box<dyn Error>> {
        if self.current_token() == Some(&expected) {
            self.position += 1;
            Ok(())
        } else {
            Err(format!("Unexpected token: expected {:?}, found {:?}", expected, self.current_token()).into())
        }
    }

    // Consume an identifier token
    fn consume_identifier(&mut self) -> Result<String, Box<dyn Error>> {
        if let Some(Token::Identifier(name)) = self.current_token().cloned() {
            self.position += 1;
            Ok(name)
        } else {
            Err(format!("Expected identifier, found {:?}", self.current_token()).into())
        }
    }

    // Consume a string token
    fn consume_string(&mut self) -> Result<String, Box<dyn Error>> {
        if let Some(Token::String(value)) = self.current_token().cloned() {
            self.position += 1;
            Ok(value)
        } else {
            Err(format!("Expected string, found {:?}", self.current_token()).into())
        }
    }

    // Get the current token
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }
}

// Compiler for converting source code into opcodes
pub struct CSCLCompiler {
    lexer: Lexer,
}

impl CSCLCompiler {
    // Create a new compiler with the given input source code
    pub fn new(input: &str) -> Self {
        CSCLCompiler {
            lexer: Lexer::new(input),
        }
    }

    // Compile the source code into a vector of opcodes
    pub fn compile(&mut self) -> Result<Vec<Opcode>, Box<dyn Error>> {
        let tokens = self.lexer.tokens();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }
}

impl Lexer {
    // Get all tokens from the input
    fn tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let input = "function test(x, y) { return x + y; }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokens();

        assert_eq!(tokens, vec![
            Token::Function,
            Token::Identifier("test".to_string()),
            Token::LParen,
            Token::Identifier("x".to_string()),
            Token::Comma,
            Token::Identifier("y".to_string()),
            Token::RParen,
            Token::LBrace,
            Token::Return,
            Token::Identifier("x".to_string()),
            Token::Plus,
            Token::Identifier("y".to_string()),
            Token::Semicolon,
            Token::RBrace,
        ]);
    }

    #[test]
    fn test_compiler() {
        let input = "x = 5 + 3 * 2; y = (10 - 4) / 2;";
        let mut compiler = CSCLCompiler::new(input);
        let opcodes = compiler.compile().unwrap();

        assert_eq!(opcodes, vec![
            Opcode::Push(Value::Int(5)),
            Opcode::Push(Value::Int(3)),
            Opcode::Push(Value::Int(2)),
            Opcode::Mul,
            Opcode::Add,
            Opcode::Store("x".to_string()),
            Opcode::Push(Value::Int(10)),
            Opcode::Push(Value::Int(4)),
            Opcode::Sub,
            Opcode::Push(Value::Int(2)),
            Opcode::Div,
            Opcode::Store("y".to_string()),
        ]);
    }
}
