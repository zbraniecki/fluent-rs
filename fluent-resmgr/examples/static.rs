use fluent_resmgr::FluentResourceManager;
use unic_langid::langid;

#[tokio::main]
async fn main() {
    let res_mgr = FluentResourceManager::new(
        "./tests/resources/{locale}/{res_id}".to_string(),
        vec![langid!("en-US")],
    );

    let loc = res_mgr.create_localization(vec!["test.ftl".to_string()], false);

    let mut errors = vec![];

    let value = loc.format_value("hello-world", None, &mut errors).await;

    assert_eq!(value, "Hello World");
}
