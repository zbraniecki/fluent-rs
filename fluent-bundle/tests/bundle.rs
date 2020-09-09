use fluent_bundle::{FluentBundle, FluentResource};
use fluent_syntax::ast;
use fluent_syntax::parser::Parser;

#[test]
fn test_bundle_mix() {
    let mut bundle = FluentBundle::new();

    let res = FluentResource::try_new("key=Value".to_string()).expect("");

    let ast: ast::Resource<String> = ast::Resource { body: vec![] };
    let res2 = FluentResource::Owned(ast);
    bundle.add_resource(res);
    bundle.add_resource(res2);
}
