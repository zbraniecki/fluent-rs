use crate::manager::FluentResourceManager;
use crate::manager::{ErrorReporter, LocalesProvider};
use std::rc::Rc;
use unic_langid::LanguageIdentifier;

pub struct InnerStaticProvider {
    locales: Vec<LanguageIdentifier>,
}
#[derive(Clone)]
pub struct StaticProvider {
    inner: Rc<InnerStaticProvider>,
}

impl LocalesProvider for StaticProvider {
    fn locales(&self) -> <Vec<LanguageIdentifier> as IntoIterator>::IntoIter {
        self.inner.locales.clone().into_iter()
    }
}

impl ErrorReporter for StaticProvider {
    fn report_errors<E: std::error::Error>(&self, errors: Vec<E>) {
        for error in errors {
            println!("Error: {}", error);
        }
    }
}

impl FluentResourceManager<StaticProvider> {
    pub fn new(path_scheme: String, locales: Vec<LanguageIdentifier>) -> Self {
        Self::with_provider(
            path_scheme,
            StaticProvider {
                inner: Rc::new(InnerStaticProvider { locales }),
            },
        )
    }
}
