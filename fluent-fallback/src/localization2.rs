use std::borrow::Cow;
// use crate::generator::{BundleGenerator, FluentBundleResult};
// use unic_langid::LanguageIdentifier;
use futures::{ready, Stream};
use std::future::Future;

pub struct ResourceManager {
    pub resources: Vec<(String, String)>,
}

impl ResourceManager {
    pub fn get_resource(&self, id: &str) -> Option<&String> {
        self.resources
            .iter()
            .find(|r| r.0 == id)
            .map(|(id, res)| res)
    }
}

pub struct Localization2<S>
where
    S: StreamProvider,
{
    bundles: S::BundlesStream,
    res_ids: Vec<String>,
}

impl<S> Localization2<S>
where
    S: StreamProvider,
{
    pub fn new(res_ids: Vec<String>, provider: S) -> Self {
        // let locales = vec!["en".parse().unwrap()];
        Self {
            bundles: provider.bundles_stream(res_ids.clone()),
            res_ids,
        }
    }

    pub fn add_resource_id(&mut self, res_id: String) {
        self.res_ids.push(res_id);
    }

    pub fn format_value<'l>(&self, id: &str) -> impl Future<Output = Option<Cow<'l, str>>> {
        async {
            use futures::StreamExt;
            let mut bundle_stream = self.bundles;
            while let Some(bundle) = bundle_stream.next().await {}
            None
        }
    }
}

pub trait StreamProvider {
    type BundlesStream: Stream<Item = String>;

    fn bundles_stream(&self, res_ids: Vec<String>) -> Self::BundlesStream {
        panic!();
    }
}

impl StreamProvider for ResourceManager {
    type BundlesStream = BundleStream;
}

pub struct BundleStream {}

impl Stream for BundleStream {
    type Item = String;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        todo!()
    }
}

// fn bundles_stream<B>() -> B
// where B: Iterator<Item = FluentResource> {
//     std::iter::from_fn(|| {
//         None
//     })
// }
