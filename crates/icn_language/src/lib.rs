use icn_common::{Opcode, Value};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alpha1, alphanumeric1, char, multispace0, multispace1},
    combinator::{map, recognize, opt},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
};

#[derive(Debug, PartialEq)]
pub enum Statement {
    Net_Node_Connect { node1: String, node2: String },
    Chain_Block_Create { transactions: Vec<String> },
    Econ_Currency_Mint { amount: f64, currency_type: String },
    Gov_Proposal_Submit { description: String },
    Coop_Member_Add { coop_id: String, member_id: String },
    Comm_Event_Organize { event_details: String },
    Vote_On_Proposal { proposal_id: String, vote: bool },
    Allocate_Resource { resource: String, amount: i64 },
    Update_Reputation { address: String, change: i64 },
    Create_Proposal { title: String, description: String },
    Get_Proposal_Status { proposal_id: String },
    Emit_Event { event_name: String, event_data: String },
}

fn parse_string(input: &str) -> IResult<&str, String> {
    map(
        delimited(
            char('"'),
            take_while1(|c| c != '"'),
            char('"')
        ),
        |s: &str| s.to_string()
    )(input)
}

fn parse_number(input: &str) -> IResult<&str, f64> {
    map(
        recognize(tuple((
            opt(char('-')),
            take_while1(|c: char| c.is_digit(10)),
            opt(pair(
                char('.'),
                take_while1(|c: char| c.is_digit(10))
            ))
        ))),
        |s: &str| s.parse().unwrap()
    )(input)
}

fn parse_integer(input: &str) -> IResult<&str, i64> {
    map(
        recognize(pair(
            opt(char('-')),
            take_while1(|c: char| c.is_digit(10))
        )),
        |s: &str| s.parse().unwrap()
    )(input)
}

fn parse_boolean(input: &str) -> IResult<&str, bool> {
    alt((
        map(tag("true"), |_| true),
        map(tag("false"), |_| false)
    ))(input)
}

fn parse_net_node_connect(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("net-node-connect"),
            multispace0,
            char('('),
            multispace0,
            parse_string,
            multispace0,
            char(','),
            multispace0,
            parse_string,
            multispace0,
            char(')')
        )),
        |(_, _, _, _, node1, _, _, _, node2, _, _)| Statement::Net_Node_Connect { node1, node2 }
    )(input)
}

fn parse_chain_block_create(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("chain-block-create"),
            multispace0,
            char('('),
            multispace0,
            separated_list0(tuple((multispace0, char(','), multispace0)), parse_string),
            multispace0,
            char(')')
        )),
        |(_, _, _, _, transactions, _, _)| Statement::Chain_Block_Create { transactions }
    )(input)
}

fn parse_econ_currency_mint(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("econ-currency-mint"),
            multispace0,
            char('('),
            multispace0,
            parse_number,
            multispace0,
            char(','),
            multispace0,
            parse_string,
            multispace0,
            char(')')
        )),
        |(_, _, _, _, amount, _, _, _, currency_type, _, _)| Statement::Econ_Currency_Mint { amount, currency_type }
    )(input)
}

fn parse_gov_proposal_submit(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("gov-proposal-submit"),
            multispace0,
            char('('),
            multispace0,
            parse_string,
            multispace0,
            char(')')
        )),
        |(_, _, _, _, description, _, _)| Statement::Gov_Proposal_Submit { description }
    )(input)
}

fn parse_coop_member_add(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("coop-member-add"),
            multispace0,
            char('('),
            multispace0,
            parse_string,
            multispace0,
            char(','),
            multispace0,
            parse_string,
            multispace0,
            char(')')
        )),
        |(_, _, _, _, coop_id, _, _, _, member_id, _, _)| Statement::Coop_Member_Add { coop_id, member_id }
    )(input)
}

fn parse_comm_event_organize(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("comm-event-organize"),
            multispace0,
            char('('),
            multispace0,
            parse_string,
            multispace0,
            char(')')
        )),
        |(_, _, _, _, event_details, _, _)| Statement::Comm_Event_Organize { event_details }
    )(input)
}

fn parse_vote_on_proposal(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("vote-on-proposal"),
            multispace0,
            char('('),
            multispace0,
            parse_string,
            multispace0,
            char(','),
            multispace0,
            parse_boolean,
            multispace0,
            char(')')
        )),
        |(_, _, _, _, proposal_id, _, _, _, vote, _, _)| Statement::Vote_On_Proposal { proposal_id, vote }
    )(input)
}

fn parse_allocate_resource(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("allocate-resource"),
            multispace0,
            char('('),
            multispace0,
            parse_string,
            multispace0,
            char(','),
            multispace0,
            parse_integer,
            multispace0,
            char(')')
        )),
        |(_, _, _, _, resource, _, _, _, amount, _, _)| Statement::Allocate_Resource { resource, amount }
    )(input)
}

fn parse_update_reputation(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("update-reputation"),
            multispace0,
            char('('),
            multispace0,
            parse_string,
            multispace0,
            char(','),
            multispace0,
            parse_integer,
            multispace0,
            char(')')
        )),
        |(_, _, _, _, address, _, _, _, change, _, _)| Statement::Update_Reputation { address, change }
    )(input)
}

fn parse_create_proposal(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("create-proposal"),
            multispace0,
            char('('),
            multispace0,
            parse_string,
            multispace0,
            char(','),
            multispace0,
            parse_string,
            multispace0,
            char(')')
        )),
        |(_, _, _, _, title, _, _, _, description, _, _)| Statement::Create_Proposal { title, description }
    )(input)
}

fn parse_get_proposal_status(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("get-proposal-status"),
            multispace0,
            char('('),
            multispace0,
            parse_string,
            multispace0,
            char(')')
        )),
        |(_, _, _, _, proposal_id, _, _)| Statement::Get_Proposal_Status { proposal_id }
    )(input)
}

fn parse_emit_event(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("emit-event"),
            multispace0,
            char('('),
            multispace0,
            parse_string,
            multispace0,
            char(','),
            multispace0,
            parse_string,
            multispace0,
            char(')')
        )),
        |(_, _, _, _, event_name, _, _, _, event_data, _, _)| Statement::Emit_Event { event_name, event_data }
    )(input)
}

fn parse_statement(input: &str) -> IResult<&str, Statement> {
    alt((
        parse_net_node_connect,
        parse_chain_block_create,
        parse_econ_currency_mint,
        parse_gov_proposal_submit,
        parse_coop_member_add,
        parse_comm_event_organize,
        parse_vote_on_proposal,
        parse_allocate_resource,
        parse_update_reputation,
        parse_create_proposal,
        parse_get_proposal_status,
        parse_emit_event
    ))(input)
}

pub fn compile(source: &str) -> Result<Vec<Statement>, String> {
    let mut statements = Vec::new();
    let mut remaining = source.trim();

    while !remaining.is_empty() {
        match parse_statement(remaining) {
            Ok((rest, statement)) => {
                statements.push(statement);
                remaining = rest.trim();
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                return Err(format!("Failed to parse statement: {:?}", e));
            }
            Err(_) => return Err("Unknown parsing error".to_string()),
        }
    }

    Ok(statements)
}

pub fn generate_bytecode(statements: &[Statement]) -> Vec<Opcode> {
    let mut bytecode = Vec::new();

    for statement in statements {
        match statement {
            Statement::Net_Node_Connect { node1, node2 } => {
                bytecode.push(Opcode::Push(Value::String(node1.clone())));
                bytecode.push(Opcode::Push(Value::String(node2.clone())));
                bytecode.push(Opcode::NetNodeConnect);
            },
            Statement::Chain_Block_Create { transactions } => {
                for tx in transactions {
                    bytecode.push(Opcode::Push(Value::String(tx.clone())));
                }
                bytecode.push(Opcode::Push(Value::Int(transactions.len() as i64)));
                bytecode.push(Opcode::ChainBlockCreate);
            },
            Statement::Econ_Currency_Mint { amount, currency_type } => {
                bytecode.push(Opcode::Push(Value::Float(*amount)));
                bytecode.push(Opcode::Push(Value::String(currency_type.clone())));
                bytecode.push(Opcode::EconCurrencyMint);
            },
            Statement::Gov_Proposal_Submit { description } => {
                bytecode.push(Opcode::Push(Value::String(description.clone())));
                bytecode.push(Opcode::GovProposalSubmit);
            },
            Statement::Coop_Member_Add { coop_id, member_id } => {
                bytecode.push(Opcode::Push(Value::String(coop_id.clone())));
                bytecode.push(Opcode::Push(Value::String(member_id.clone())));
                bytecode.push(Opcode::CoopMemberAdd);
            },
            Statement::Comm_Event_Organize { event_details } => {
                bytecode.push(Opcode::Push(Value::String(event_details.clone())));
                bytecode.push(Opcode::CommEventOrganize);
            },
            Statement::Vote_On_Proposal { proposal_id, vote } => {
                bytecode.push(Opcode::Push(Value::String(proposal_id.clone())));
                bytecode.push(Opcode::Push(Value::Bool(*vote)));
                bytecode.push(Opcode::VoteOnProposal);
            },
            Statement::Allocate_Resource { resource, amount } => {
                bytecode.push(Opcode::Push(Value::String(resource.clone())));
                bytecode.push(Opcode::Push(Value::Int(*amount)));
                bytecode.push(Opcode::AllocateResource);
            },
            Statement::Update_Reputation { address, change } => {
                bytecode.push(Opcode::Push(Value::String(address.clone())));
                bytecode.push(Opcode::Push(Value::Int(*change)));
                bytecode.push(Opcode::UpdateReputation);
            },
            Statement::Create_Proposal { title, description } => {
                bytecode.push(Opcode::Push(Value::String(title.clone())));
                bytecode.push(Opcode::Push(Value::String(description.clone())));
                bytecode.push(Opcode::CreateProposal);
            },
            Statement::Get_Proposal_Status { proposal_id } => {
                bytecode.push(Opcode::Push(Value::String(proposal_id.clone())));
                bytecode.push(Opcode::GetProposalStatus);
            },
            Statement::Emit_Event { event_name, event_data } => {
                bytecode.push(Opcode::Push(Value::String(event_name.clone())));
                bytecode.push(Opcode::Push(Value::String(event_data.clone())));
                bytecode.push(Opcode::EmitEvent(event_name.clone()));
            },
        }
    }

    bytecode
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_net_node_connect() {
        let input = r#"net-node-connect("node1", "node2")"#;
        let result = parse_net_node_connect(input);
        assert!(result.is_ok());
        let (_, statement) = result.unwrap();
        assert_eq!(statement, Statement::Net_Node_Connect {
            node1: "node1".to_string(),
            node2: "node2".to_string(),
        });
    }

    #[test]
    fn test_parse_chain_block_create() {
        let input = r#"chain-block-create("tx1", "tx2", "tx3")"#;
        let result = parse_chain_block_create(input);
        assert!(result.is_ok());
        let (_, statement) = result.unwrap();
        assert_eq!(statement, Statement::Chain_Block_Create {
            transactions: vec!["tx1".to_string(), "tx2".to_string(), "tx3".to_string()],
        });
    }

    #[test]
    fn test_parse_econ_currency_mint() {
        let input = r#"econ-currency-mint(100.5, "BasicNeeds")"#;
        let result = parse_econ_currency_mint(input);
        assert!(result.is_ok());
        let (_, statement) = result.unwrap();
        assert_eq!(statement, Statement::Econ_Currency_Mint {
            amount: 100.5,
            currency_type: "BasicNeeds".to_string(),
        });
    }

    #[test]
    fn test_parse_gov_proposal_submit() {
        let input = r#"gov-proposal-submit("Increase node count")"#;
        let result = parse_gov_proposal_submit(input);
        assert!(result.is_ok());
        let (_, statement) = result.unwrap();
        assert_eq!(statement, Statement::Gov_Proposal_Submit {
            description: "Increase node count".to_string(),
        });
    }

    #[test]
    fn test_parse_coop_member_add() {
        let input = r#"coop-member-add("coop1", "member1")"#;
        let result = parse_coop_member_add(input);
        assert!(result.is_ok());
        let (_, statement) = result.unwrap();
        assert_eq!(statement, Statement::Coop_Member_Add {
            coop_id: "coop1".to_string(),
            member_id: "member1".to_string(),
        });
    }

    #[test]
    fn test_parse_comm_event_organize() {
        let input = r#"comm-event-organize("Community meetup on Saturday")"#;
        let result = parse_comm_event_organize(input);
        assert!(result.is_ok());
        let (_, statement) = result.unwrap();
        assert_eq!(statement, Statement::Comm_Event_Organize {
            event_details: "Community meetup on Saturday".to_string(),
        });
    }

    #[test]
    fn test_parse_vote_on_proposal() {
        let input = r#"vote-on-proposal("proposal1", true)"#;
        let result = parse_vote_on_proposal(input);
        assert!(result.is_ok());
        let (_, statement) = result.unwrap();
        assert_eq!(statement, Statement::Vote_On_Proposal {
            proposal_id: "proposal1".to_string(),
            vote: true,
        });
    }

    #[test]
    fn test_parse_allocate_resource() {
        let input = r#"allocate-resource("computing_power", 100)"#;
        let result = parse_allocate_resource(input);
        assert!(result.is_ok());
        let (_, statement) = result.unwrap();
        assert_eq!(statement, Statement::Allocate_Resource {
            resource: "computing_power".to_string(),
            amount: 100,
        });
    }

    #[test]
    fn test_parse_update_reputation() {
        let input = r#"update-reputation("user1", 5)"#;
        let result = parse_update_reputation(input);
        assert!(result.is_ok());
        let (_, statement) = result.unwrap();
        assert_eq!(statement, Statement::Update_Reputation {
            address: "user1".to_string(),
            change: 5,
        });
    }

    #[test]
    fn test_parse_create_proposal() {
        let input = r#"create-proposal("New Policy", "Implement resource sharing")"#;
        let result = parse_create_proposal(input);
        assert!(result.is_ok());
        let (_, statement) = result.unwrap();
        assert_eq!(statement, Statement::Create_Proposal {
            title: "New Policy".to_string(),
            description: "Implement resource sharing".to_string(),
        });
    }

    #[test]
    fn test_parse_get_proposal_status() {
        let input = r#"get-proposal-status("proposal1")"#;
        let result = parse_get_proposal_status(input);
        assert!(result.is_ok());
        let (_, statement) = result.unwrap();
        assert_eq!(statement, Statement::Get_Proposal_Status {
            proposal_id: "proposal1".to_string(),
        });
    }

    #[test]
    fn test_parse_emit_event() {
        let input = r#"emit-event("NewMember", "Alice joined the network")"#;
        let result = parse_emit_event(input);
        assert!(result.is_ok());
        let (_, statement) = result.unwrap();
        assert_eq!(statement, Statement::Emit_Event {
            event_name: "NewMember".to_string(),
            event_data: "Alice joined the network".to_string(),
        });
    }

    #[test]
    fn test_compile_multiple_statements() {
        let input = r#"
            net-node-connect("node1", "node2")
            econ-currency-mint(100.0, "BasicNeeds")
            gov-proposal-submit("Increase node count")
            coop-member-add("coop1", "Alice")
            comm-event-organize("Community meetup")
            vote-on-proposal("proposal1", true)
            allocate-resource("computing_power", 50)
            update-reputation("Bob", 10)
            create-proposal("New Policy", "Implement resource sharing")
            get-proposal-status("proposal2")
            emit-event("NetworkUpdate", "New node added")
        "#;
        let result = compile(input);
        assert!(result.is_ok());
        let statements = result.unwrap();
        assert_eq!(statements.len(), 11);
        assert!(matches!(statements[0], Statement::Net_Node_Connect { .. }));
        assert!(matches!(statements[1], Statement::Econ_Currency_Mint { .. }));
        assert!(matches!(statements[2], Statement::Gov_Proposal_Submit { .. }));
        assert!(matches!(statements[3], Statement::Coop_Member_Add { .. }));
        assert!(matches!(statements[4], Statement::Comm_Event_Organize { .. }));
        assert!(matches!(statements[5], Statement::Vote_On_Proposal { .. }));
        assert!(matches!(statements[6], Statement::Allocate_Resource { .. }));
        assert!(matches!(statements[7], Statement::Update_Reputation { .. }));
        assert!(matches!(statements[8], Statement::Create_Proposal { .. }));
        assert!(matches!(statements[9], Statement::Get_Proposal_Status { .. }));
        assert!(matches!(statements[10], Statement::Emit_Event { .. }));
    }

    #[test]
    fn test_compile_with_whitespace() {
        let input = r#"
            net-node-connect("node1", "node2")
            
            econ-currency-mint(100.0, "BasicNeeds")
                gov-proposal-submit("Increase node count")
            
        "#;
        let result = compile(input);
        assert!(result.is_ok());
        let statements = result.unwrap();
        assert_eq!(statements.len(), 3);
    }

    #[test]
    fn test_compile_error() {
        let input = r#"
            net-node-connect("node1", "node2")
            invalid-statement()
            econ-currency-mint(100.0, "BasicNeeds")
        "#;
        let result = compile(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_bytecode() {
        let statements = vec![
            Statement::Net_Node_Connect {
                node1: "node1".to_string(),
                node2: "node2".to_string(),
            },
            Statement::Econ_Currency_Mint {
                amount: 100.0,
                currency_type: "BasicNeeds".to_string(),
            },
        ];

        let bytecode = generate_bytecode(&statements);
        
        assert_eq!(bytecode.len(), 5);
        assert!(matches!(bytecode[0], Opcode::Push(Value::String(_))));
        assert!(matches!(bytecode[1], Opcode::Push(Value::String(_))));
        assert!(matches!(bytecode[2], Opcode::NetNodeConnect));
        assert!(matches!(bytecode[3], Opcode::Push(Value::Float(_))));
        assert!(matches!(bytecode[4], Opcode::Push(Value::String(_))));
    }
}