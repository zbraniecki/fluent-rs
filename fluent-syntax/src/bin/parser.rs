use fluent_syntax::parser::Parser;
use std::env;
use std::fs;
use std::io;

fn read_file(path: &str) -> Result<String, io::Error> {
    fs::read_to_string(path)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let source = read_file(args.get(1).expect("Pass an argument")).expect("Failed to fetch file");

    let parser = Parser::new(&source);
    let ast = parser.parse().unwrap();

    #[cfg(feature = "json")]
    {
        use fluent_syntax::json;
        let target_json = json::serialize_to_pretty_json(&ast).unwrap();
        println!("{}", target_json);
    }
    #[cfg(not(feature = "json"))]
    {
        use std::fmt::Write;
        let mut result = String::new();
        write!(result, "{:#?}", ast).unwrap();
        println!("{}", result);
    }
}
