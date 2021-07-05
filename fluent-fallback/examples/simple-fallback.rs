use fluent_fallback::localization2::*;

#[tokio::main]
async fn main() {
    let res_mgr = ResourceManager {
        resources: vec![
            ("main.ftl".to_string(), "key = Value".to_string()),
            ("menu.ftl".to_string(), "key2 = Value 2".to_string()),
        ],
    };

    let res_ids = vec!["main.ftl".to_string()];

    let mut loc = Localization2::new(res_ids, res_mgr);

    let future = loc.format_value("key");

    loc.add_resource_id("menu.ftl".to_string());

    future.await;
}
