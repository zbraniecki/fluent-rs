#![feature(async_closure)]

use async_std::{fs::File, io, prelude::*};
use elsa::FrozenMap;
use fluent_bundle::FluentBundle;
use fluent_bundle::FluentResource;
use fluent_fallback::asynchronous::Localization;
use unic_langid::langid;

use std::cell::RefCell;
use std::iter;

async fn read_file(path: String) -> io::Result<String> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}

#[test]
fn localization_async_format() {
    let resource_ids: Vec<String> = vec!["test.ftl".into(), "test2.ftl".into()];
    let res_path_scheme = "./tests/resources/{locale}/{res_id}";
    let locales = vec![langid!("pl"), langid!("en-US")];

    let generate_messages = |res_ids: &[String]| {
        let mut locales = locales.iter();
        let res_ids = res_ids.to_vec();

        iter::from_fn(move || {
            let res_ids = res_ids.clone();
            locales.next().map(async move |locale| {
                let mut bundle = FluentBundle::new(vec![locale]);
                let res_path = res_path_scheme.replace("{locale}", &locale.to_string());

                for res_id in res_ids.clone() {
                    let path = res_path.replace("{res_id}", &res_id);
                    let source = read_file(path.clone()).await.unwrap();
                    let res = FluentResource::try_new(source).unwrap();
                    bundle.add_resource(res).unwrap();
                }
                bundle
            })
        })
    };

    let loc = Localization::new(resource_ids, generate_messages);

    // let value = loc.format_value("hello-world", None);
    // assert_eq!(value, "Hello World [pl]");
    //
    // let value = loc.format_value("missing-message", None);
    // assert_eq!(value, "missing-message");
    //
    // let value = loc.format_value("hello-world-3", None);
    // assert_eq!(value, "Hello World 3 [en]");
}

// #[test]
// fn localization_on_change() {
//     let resources: FrozenMap<String, Box<FluentResource>> = FrozenMap::new();
//
//     let resource_ids: Vec<String> = vec!["test.ftl".into(), "test2.ftl".into()];
//     let res_path_scheme = "./tests/resources/{locale}/{res_id}";
//
//     let available_locales = RefCell::new(vec![langid!("en-US")]);
//
//     let generate_messages = |res_ids: &[String]| {
//         let mut bundles = vec![];
//
//         for locale in available_locales.borrow().iter() {
//             let mut bundle = FluentBundle::new(vec![locale]);
//             let res_path = res_path_scheme.replace("{locale}", &locale.to_string());
//             for res_id in res_ids {
//                 let path = res_path.replace("{res_id}", res_id);
//                 let res = if let Some(res) = resources.get(&path) {
//                     res
//                 } else {
//                     let source = read_file(&path).unwrap();
//                     let res = FluentResource::try_new(source).unwrap();
//                     resources.insert(path, Box::new(res))
//                 };
//                 bundle.add_resource(res).unwrap();
//             }
//             bundles.push(bundle);
//         }
//
//         return bundles.into_iter();
//     };
//
//     let mut loc = Localization::new(resource_ids, generate_messages);
//
//     let value = loc.format_value("hello-world", None);
//     assert_eq!(value, "Hello World [en]");
//
//     available_locales.borrow_mut().insert(0, langid!("pl"));
//
//     loc.on_change();
//
//     let value = loc.format_value("hello-world", None);
//     assert_eq!(value, "Hello World [pl]");
// }
