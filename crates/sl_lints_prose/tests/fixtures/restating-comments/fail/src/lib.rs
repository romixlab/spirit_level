//! Fixture.

pub struct Parser;

impl Parser {
    pub fn new() -> Parser {
        Parser
    }
}

pub fn run(counter: &mut u32) -> Parser {
    // increment the counter
    *counter += 1;
    // create a new parser
    Parser::new()
}
