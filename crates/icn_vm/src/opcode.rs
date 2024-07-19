#[derive(Debug, Clone, PartialEq, PartialOrd)] // Add PartialOrd here
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
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
    Return,
    Store(String),
    Load(String),
    Call(String),
    Vote(String),
    AllocateResource(String),
    UpdateReputation(String),
    CreateProposal,
    GetProposalStatus,
    Emit(String),
}
