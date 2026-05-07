use anyhow::{anyhow, Result};
use winnow::{
    ascii::multispace0,
    combinator::delimited,
    token::take_while,
    Parser,
};


#[derive(Debug)]
pub enum ParsedGetCommand {
    All,
    Key(String),
}


fn keyword_ic<'a>(kw: &'static str) -> impl Parser<&'a str, &'a str, winnow::error::ContextError> {
    take_while(1.., |c: char| c.is_alphabetic())
        .verify(move |s: &str| s.eq_ignore_ascii_case(kw))
}

fn quoted_str<'a>() -> impl Parser<&'a str, &'a str, winnow::error::ContextError> {
    delimited('"', take_while(0.., |c| c != '"'), '"')
}

fn swallow_ws(input: &mut &str) {
    let _ = multispace0::<_, winnow::error::ContextError>.parse_next(input);
}

pub fn parse_put_command(command: &str) -> Result<Vec<(String, String)>> {
    let mut input = command.trim();

    keyword_ic("PUT")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid PUT command: expected PUT keyword"))?;

    swallow_ws(&mut input);

    let mut pairs = Vec::new();
    loop {
        swallow_ws(&mut input);
        let key = quoted_str()
            .parse_next(&mut input)
            .map_err(|_| anyhow!("Invalid PUT command: expected quoted key"))?;

        swallow_ws(&mut input);
        if !input.starts_with(':') {
            return Err(anyhow!("Invalid PUT command: expected ':' separator"));
        }
        input = &input[1..];

        swallow_ws(&mut input);
        let value = quoted_str()
            .parse_next(&mut input)
            .map_err(|_| anyhow!("Invalid PUT command: expected quoted value"))?;

        pairs.push((key.to_string(), value.to_string()));

        swallow_ws(&mut input);
        if input.starts_with(',') {
            input = &input[1..];
            continue;
        }
        break;
    }

    swallow_ws(&mut input);
    if !input.is_empty() {
        return Err(anyhow!("Invalid PUT command: unexpected trailing input"));
    }
    if pairs.is_empty() {
        return Err(anyhow!("Invalid PUT command: no key-value pairs found"));
    }

    Ok(pairs)
}


pub fn parse_get_command(command: &str) -> Result<ParsedGetCommand> {
    let mut input = command.trim();

    keyword_ic("GET")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid GET command"))?;
    swallow_ws(&mut input);
    keyword_ic("WHERE")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid GET command: expected WHERE"))?;
    swallow_ws(&mut input);
    keyword_ic("KEY")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid GET command: expected KEY"))?;
    swallow_ws(&mut input);

    if !input.starts_with('=') {
        return Err(anyhow!("Invalid GET command: expected '='"));
    }
    input = &input[1..];
    swallow_ws(&mut input);

    if input.starts_with('*') {
        input = &input[1..];
        swallow_ws(&mut input);
        if !input.is_empty() {
            return Err(anyhow!("Invalid GET command: unexpected trailing input after '*'"));
        }
        return Ok(ParsedGetCommand::All);
    }

    let key = quoted_str()
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid GET command: expected quoted key or '*'"))?;

    swallow_ws(&mut input);
    if !input.is_empty() {
        return Err(anyhow!("Invalid GET command: unexpected trailing input"));
    }

    Ok(ParsedGetCommand::Key(key.to_string()))
}


pub fn parse_delete_command(command: &str) -> Result<String> {
    let mut input = command.trim();

    keyword_ic("DEL")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid DELETE command: expected DEL"))?;
    swallow_ws(&mut input);
    keyword_ic("WHERE")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid DELETE command: expected WHERE"))?;
    swallow_ws(&mut input);
    keyword_ic("KEY")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid DELETE command: expected KEY"))?;
    swallow_ws(&mut input);

    if !input.starts_with('=') {
        return Err(anyhow!("Invalid DELETE command: expected '='"));
    }
    input = &input[1..];
    swallow_ws(&mut input);

    let key = quoted_str()
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid DELETE command: expected quoted key"))?;

    swallow_ws(&mut input);
    if !input.is_empty() {
        return Err(anyhow!("Invalid DELETE command: unexpected trailing input"));
    }

    Ok(key.to_string())
}


pub fn parse_identifier_get(command: &str) -> Result<()> {
    let mut input = command.trim();

    keyword_ic("IDENTIFIER")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid IDENTIFIER command"))?;
    swallow_ws(&mut input);
    keyword_ic("GET")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid IDENTIFIER command: expected GET"))?;

    swallow_ws(&mut input);
    if !input.is_empty() {
        return Err(anyhow!("Invalid IDENTIFIER GET command: unexpected trailing input"));
    }

    Ok(())
}


pub fn parse_identifier_set(command: &str) -> Result<String> {
    let mut input = command.trim();

    keyword_ic("IDENTIFIER")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid IDENTIFIER command"))?;
    swallow_ws(&mut input);
    keyword_ic("SET")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid IDENTIFIER command: expected SET"))?;
    swallow_ws(&mut input);

    let id = quoted_str()
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid IDENTIFIER SET command: expected quoted identifier"))?;

    swallow_ws(&mut input);
    if !input.is_empty() {
        return Err(anyhow!("Invalid IDENTIFIER SET command: unexpected trailing input"));
    }

    Ok(id.to_string())
}


pub fn parse_compact(command: &str) -> Result<()> {
    let mut input = command.trim();

    keyword_ic("COMPACT")
        .parse_next(&mut input)
        .map_err(|_| anyhow!("Invalid COMPACT command"))?;

    swallow_ws(&mut input);
    if !input.is_empty() {
        return Err(anyhow!("Invalid COMPACT command: unexpected trailing input"));
    }

    Ok(())
}
