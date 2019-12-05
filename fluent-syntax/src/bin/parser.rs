use fluent_syntax::parser::Parser;
use std::fmt::Write;
use std::io::Read;

fn main() {
    let input = include_str!("../../benches/simple.ftl");

    let parser = Parser::new(input.as_bytes().bytes());

    let ast: Result<Vec<_>, _> = parser.collect();

    let mut result = String::new();
    write!(result, "{:#?}", ast).unwrap();
    println!("{}", result);
}
