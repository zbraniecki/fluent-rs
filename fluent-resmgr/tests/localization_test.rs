use fluent_fallback::generator::BundleGenerator;
use fluent_fallback::Localization;
use fluent_resmgr::FluentResourceManager;
use unic_langid::langid;

#[test]
fn localization_format_value() {
    let res_mgr = FluentResourceManager::new(
        "./tests/resources/{locale}/{res_id}".to_string(),
        vec![langid!("en-US"), langid!("pl")],
    );

    let loc = Localization::with_generator(vec!["test.ftl".into()], true, res_mgr);
    let mut errors = vec![];

    let value = loc.format_value_sync("hello-world", None, &mut errors);
    assert_eq!(value, "Hello World");

    let value2 = loc.format_value_sync("new-message", None, &mut errors);
    assert_eq!(value2, "Nowa Wiadomość");

    let value3 = loc.format_value_sync("missing-message", None, &mut errors);
    assert_eq!(value3, "missing-message");

    assert_eq!(errors.len(), 1);
}

#[test]
fn resmgr_get_bundle() {
    let res_mgr = FluentResourceManager::new(
        "./tests/resources/{locale}/{res_id}".to_string(),
        vec![langid!("en-US"), langid!("pl")],
    );

    let bundle = res_mgr
        .bundles_iter(vec!["test.ftl".into()])
        .next()
        .unwrap();

    let mut errors = vec![];
    let msg = bundle.get_message("hello-world").expect("Message exists");
    let pattern = msg.value.expect("Message has a value");
    let value = bundle.format_pattern(&pattern, None, &mut errors);
    assert_eq!(value, "Hello World");
    assert_eq!(errors.len(), 0);
}
