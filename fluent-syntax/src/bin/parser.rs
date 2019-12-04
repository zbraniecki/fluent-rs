use fluent_syntax::parser::Parser;
use std::fmt::Write;

fn main() {
    let input = include_str!("../../benches/simple.ftl");

    let parser = Parser::new(input);
    let ast = parser.parse();

    let mut result = String::new();
    write!(result, "{:#?}", ast).unwrap();
    println!("{}", result);
}
