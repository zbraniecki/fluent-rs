use fluent_bundle::FluentResource;
use fluent_fallback::localization2::*;
use std::rc::Rc;

#[tokio::main]
async fn main() {
    let res_mgr = ResourceManager {
        inner: Rc::new(InnerResMgr {
            resources: vec![
                (
                    "main.ftl".to_string(),
                    Rc::new(FluentResource::try_new("key = Value".to_string()).unwrap()),
                ),
                (
                    "menu.ftl".to_string(),
                    Rc::new(FluentResource::try_new("key2 = Value 2".to_string()).unwrap()),
                ),
            ],
        }),
    };

    let res_ids = vec!["main.ftl".to_string()];

    let mut loc = Localization2::new(res_ids, res_mgr);

    let future = loc.format_value("key");

    println!("Adding new res_id");
    loc.add_resource_id("menu.ftl".to_string());

    let future2 = loc.format_value("key");

    println!("Executing future 1");
    future.await;

    println!("Executing future 2");
    future2.await;

    loc.remove_resource_id("main.ftl");

    println!("Executing future 3");
    loc.format_value("key").await;
}
