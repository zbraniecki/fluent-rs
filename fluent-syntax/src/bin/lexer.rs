use fallible_iterator::FallibleIterator;
use fluent_syntax::parser::lexer;
use std::fmt::Write;
use std::io::Read;
use std::io;

fn main() {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    // let input = include_str!("../../benches/simple.ftl");

    // let lexer = lexer::Lexer::new(input.as_bytes().bytes());
    let lexer = lexer::Lexer::new(handle.bytes());
    let tokens: Result<Vec<_>, _> = lexer.collect();

    let mut result = String::new();
    write!(result, "{:#?}", tokens).unwrap();
    println!("{}", result);
}
