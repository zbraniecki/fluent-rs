use fluent_bundle::{FluentBundle, FluentResource, FluentValue};

#[test]
fn test_resolve() {
    let mut bundle = FluentBundle::new();

    let res = FluentResource::try_new(
        "key = Hello { $user }. You have { $emailCount } emails.".to_string(),
    )
    .expect("");

    bundle.add_resource(res);

    let msg = bundle.get_message("key").unwrap();

    let mut s = String::new();
    bundle
        .format_pattern(
            &mut s,
            msg.value.as_ref().unwrap(),
            Some(
                vec![
                    ("user", FluentValue::String("John")),
                    ("emailCount", 5.into()),
                ]
                .into(),
            ),
        )
        .unwrap();
    assert_eq!(s, "Hello John. You have 5 emails.");

    let mut s = String::new();
    bundle
        .format_pattern(
            &mut s,
            msg.value.as_ref().unwrap(),
            Some(vec![("user".to_string(), FluentValue::String("Amy".to_string()))].into()),
        )
        .unwrap();
    assert_eq!(s, "Hello Amy. You have ??? emails.");
}

#[test]
fn test_plural() {
    let mut bundle = FluentBundle::new();

    let res = FluentResource::try_new(
        r#"
key = { $emailCount ->
    [one] Hello One
   *[other] Hello Other
}
"#
        .to_string(),
    )
    .expect("");

    bundle.add_resource(res);

    let msg = bundle.get_message("key").unwrap();

    let mut s = String::new();
    bundle
        .format_pattern(
            &mut s,
            msg.value.as_ref().unwrap(),
            Some(vec![("emailCount", FluentValue::from(1))].into()),
        )
        .unwrap();
    assert_eq!(s, "Hello One");
}
