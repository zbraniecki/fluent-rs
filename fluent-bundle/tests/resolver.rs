use fluent_bundle::{FluentBundle, FluentResource};

#[test]
fn test_resolve() {
    let mut bundle = FluentBundle::new();

    let res = FluentResource::try_new("key=Value".to_string()).expect("");

    bundle.add_resource(res);

    let msg = bundle.get_message("key").unwrap();

    let mut s = String::new();
    bundle.format_pattern(&mut s, &msg.value.unwrap()).unwrap();
    assert_eq!(s, "Value");
}
