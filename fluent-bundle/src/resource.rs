use fluent_syntax::ast;
use fluent_syntax::parser::Parser;
use fluent_syntax::parser::ParserError;

rental! {
    mod rentals {
        use super::*;
        #[rental(covariant, debug)]
        pub struct FluentResource {
            string: String,
            ast: ast::Resource<&'string str>,
        }
    }
}

/// A resource containing a list of localization messages.
#[derive(Debug)]
pub enum FluentResource {
    Owned(ast::Resource<String>),
    Sliced(rentals::FluentResource),
}

impl FluentResource {
    pub fn try_new(source: String) -> Result<Self, (Self, Vec<ParserError>)> {
        let mut errors = None;
        let res = rentals::FluentResource::new(source, |s| match Parser::new(s).parse() {
            Ok(ast) => ast,
            Err((ast, err)) => {
                errors = Some(err);
                ast
            }
        });

        if let Some(errors) = errors {
            Err((Self::Sliced(res), errors))
        } else {
            Ok(Self::Sliced(res))
        }
    }

    pub fn ast(&self) -> &ast::Resource<&str> {
        match self {
            Self::Sliced(rental) => rental.all().ast,
            Self::Owned(ast) => unreachable!(),
        }
    }
}
