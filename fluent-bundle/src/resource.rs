use fluent_syntax::ast;
use fluent_syntax::parser::Parser;
use fluent_syntax::parser::ParserError;

// pub struct FluentMessage<'m, S> {
//     pub value: Option<&'m ast::Pattern<S>>,
//     pub attributes: Vec<(S, &'m ast::Pattern<S>)>,
// }

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

#[derive(Debug)]
/// A resource containing a list of localization messages.
pub enum FluentResource {
    Owned(ast::Resource<String>),
    Sliced(rentals::FluentResource),
}

trait GetMessage<'s, S> {
    fn get_message(&self, id: &'s str) -> Option<&ast::Message<S>>
    where
        S: PartialEq<&'s str>;
}

impl<'s, S> GetMessage<'s, S> for ast::Resource<S> {
    fn get_message(&self, id: &'s str) -> Option<&ast::Message<S>>
    where
        S: PartialEq<&'s str>,
    {
        for entry in &self.body {
            match entry {
                ast::ResourceEntry::Entry(entry) => match entry {
                    ast::Entry::Message(m) => {
                        if m.id.name == id {
                            return Some(m);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        None
    }
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

    // pub fn get_message<'s>(&self, id: &str) -> Option<FluentMessage<'s, &'s str>> {
    pub fn get_message<'s>(&'s self, id: &str) -> Option<ast::Message<&'s str>> {
        match self {
            Self::Owned(ast) => {
                let msg = ast.get_message(id).unwrap();
                Some(msg.borrowed())
            }
            Self::Sliced(res) => {
                let msg = res.all().ast.get_message(id).unwrap();
                Some(msg.clone())
            }
        }
    }
}
