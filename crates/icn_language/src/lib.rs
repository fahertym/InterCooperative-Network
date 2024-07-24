use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alpha1, alphanumeric1, char, multispace0},
    combinator::{map, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded, terminated},
};

#[derive(Debug, PartialEq)]
pub enum Statement {
    CoopCreate { name: String, coop_type: String },
    GovPolicyCreate { name: String, details: String },
    CommEventSchedule { name: String, date: String },
    ResourceAllocation { resource: String, amount: i64 },
    ReputationUpdate { address: String, change: i64 },
    VoteOnProposal { proposal_id: String, vote: bool },
    MorphemicOperation { function: String, args: Vec<String> },
}

fn parse_morphemic_identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alpha1,
        many0(alt((alphanumeric1, tag("-"))))
    ))(input)
}

fn parse_string(input: &str) -> IResult<&str, &str> {
    delimited(
        char('"'),
        take_while1(|c| c != '"'),
        char('"')
    )(input)
}

fn parse_morphemic_operation(input: &str) -> IResult<&str, Statement> {
    let (input, function) = parse_morphemic_identifier(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('(')(input)?;
    let (input, args) = many0(terminated(parse_string, multispace0))(input)?;
    let (input, _) = char(')')(input)?;

    Ok((input, Statement::MorphemicOperation {
        function: function.to_string(),
        args: args.into_iter().map(|s| s.to_string()).collect(),
    }))
}

pub fn parse_statement(input: &str) -> IResult<&str, Statement> {
    parse_morphemic_operation(input)
}

pub fn compile(source: &str) -> Result<Vec<Statement>, String> {
    let mut statements = Vec::new();
    let mut remaining = source;

    while !remaining.is_empty() {
        match parse_statement(remaining) {
            Ok((rest, statement)) => {
                statements.push(statement);
                remaining = rest.trim();
            }
            Err(_) => return Err("Failed to parse statement".to_string()),
        }
    }

    Ok(statements)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_morphemic_operation() {
        let input = r#"net-node-connect("node1", "node2")"#;
        let result = parse_morphemic_operation(input);
        assert!(result.is_ok());
        let (_, statement) = result.unwrap();
        assert_eq!(statement, Statement::MorphemicOperation {
            function: "net-node-connect".to_string(),
            args: vec!["node1".to_string(), "node2".to_string()],
        });
    }

    #[test]
    fn test_compile_multiple_statements() {
        let input = r#"
            econ-currency-mint("100", "BasicNeeds")
            gov-proposal-submit("New policy for resource allocation")
            coop-member-add("coop1", "member1")
        "#;
        let result = compile(input);
        assert!(result.is_ok());
        let statements = result.unwrap();
        assert_eq!(statements.len(), 3);
        assert!(matches!(statements[0], Statement::MorphemicOperation { function, .. } if function == "econ-currency-mint"));
        assert!(matches!(statements[1], Statement::MorphemicOperation { function, .. } if function == "gov-proposal-submit"));
        assert!(matches!(statements[2], Statement::MorphemicOperation { function, .. } if function == "coop-member-add"));
    }
}