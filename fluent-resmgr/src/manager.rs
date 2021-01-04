use fluent_bundle::FluentResource;
use fluent_fallback::generator::BundleGenerator;
use std::rc::Rc;
use crate::iter::BundleIter;
use crate::fetcher::SyncFileFetcher;
use unic_langid::LanguageIdentifier;

pub struct ResourceManager<F> {
    fetcher: F,
    locales: Vec<LanguageIdentifier>,
}

impl<F> ResourceManager<F> {
    pub fn new(fetcher: F, locales: Vec<LanguageIdentifier>) -> Self {
        Self { fetcher, locales }
    }
}

impl<F> BundleGenerator for ResourceManager<F> {
    type Resource = Rc<FluentResource>;
    type Iter = BundleIter;
    type Stream = BundleIter;

    fn bundles_stream(&self, resource_ids: Vec<String>) -> Self::Stream {
        todo!()
    }

    fn bundles_iter(&self, resource_ids: Vec<String>) -> Self::Iter {
        todo!()
    }
}

