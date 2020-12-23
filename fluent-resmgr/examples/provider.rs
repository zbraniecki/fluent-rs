use fluent_fallback::Localization;
use fluent_resmgr::manager::{ErrorReporter, FluentResourceManager, LocalesProvider};
use std::rc::Rc;
use unic_langid::{langid, LanguageIdentifier};

struct InnerApp {
    locales: Vec<LanguageIdentifier>,
}

#[derive(Clone)]
struct App {
    inner: Rc<InnerApp>,
}

impl LocalesProvider for App {
    fn locales(&self) -> <Vec<LanguageIdentifier> as IntoIterator>::IntoIter {
        self.inner.locales.clone().into_iter()
    }
}

impl ErrorReporter for App {
    fn report_errors<E: std::error::Error>(&self, errors: Vec<E>) {
        for error in errors {
            println!("Error: {}", error);
        }
    }
}

fn main() {
    let app = App {
        inner: Rc::new(InnerApp {
            locales: vec![langid!("en-US")],
        }),
    };

    let res_mgr = FluentResourceManager::with_provider(
        "./tests/resources/{locale}/{res_id}".to_string(),
        app.clone(),
    );

    let loc = Localization::with_generator(vec![
        "test.ftl".to_string(),
        "test.ftl".to_string(),
    ], true, res_mgr);

    let mut errors = vec![];

    let value = loc.format_value_sync("hello-world", None, &mut errors);

    assert_eq!(value, "Hello World");

    let value = loc.format_value_sync("hello-world2", None, &mut errors);
    println!("{:#?}", value);
    app.report_errors(errors);
}
