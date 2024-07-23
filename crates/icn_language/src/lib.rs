// File: crates/icn_language/src/lib.rs

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
    // Add more statement types as needed
}

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_"))))
    ))(input)
}

fn parse_string(input: &str) -> IResult<&str, &str> {
    delimited(
        char('"'),
        take_while1(|c| c != '"'),
        char('"')
    )(input)
}

fn parse_coop_create(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag("coop-create")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('(')(input)?;
    let (input, name) = preceded(tag("name:"), parse_string)(input)?;
    let (input, _) = char(',')(input)?;
    let (input, coop_type) = preceded(tag("type:"), parse_string)(input)?;
    let (input, _) = char(')')(input)?;
    
    Ok((input, Statement::CoopCreate { 
        name: name.to_string(), 
        coop_type: coop_type.to_string() 
    }))
}

// Implement other parsing functions for different statement types

pub fn parse_statement(input: &str) -> IResult<&str, Statement> {
    alt((
        parse_coop_create,
        // Add other statement parsers here
    ))(input)
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
    fn test_parse_coop_create() {
        let input = r#"coop-create(name: "Green Community Cooperative", type: "Sustainability")"#;
        let result = parse_coop_create(input);
        assert!(result.is_ok());
        let (_, statement) = result.unwrap();
        assert_eq!(statement, Statement::CoopCreate {
            name: "Green Community Cooperative".to_string(),
            coop_type: "Sustainability".to_string(),
        });
    }

    // Add more tests for other statement types and the compile function
}