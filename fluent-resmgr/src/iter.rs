use fluent_fallback::generator::{FluentBundleResult, BundleIterator, BundleStream};
use fluent_bundle::FluentResource;
use futures::stream::Stream;
use std::rc::Rc;

pub struct BundleIter {}

impl Iterator for BundleIter {
    type Item = FluentBundleResult<Rc<FluentResource>>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl BundleIterator for BundleIter {
    type Resource = Rc<FluentResource>;
}

impl Stream for BundleIter {
    type Item = FluentBundleResult<Rc<FluentResource>>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        todo!()
    }
}

impl BundleStream for BundleIter {
    type Resource = Rc<FluentResource>;
}
