use crate::cache::{AsyncCache, Cache};
use crate::generator::{BundleGenerator, FluentBundleResult};
use fluent_bundle::{FluentBundle, FluentResource};
use futures::{ready, Stream};
use once_cell::sync::OnceCell;
use std::borrow::Cow;
use std::future::Future;
use std::rc::Rc;
use unic_langid::LanguageIdentifier;

pub struct InnerResMgr {
    pub resources: Vec<(String, Rc<FluentResource>)>,
}

#[derive(Clone)]
pub struct ResourceManager {
    pub inner: Rc<InnerResMgr>,
}

impl ResourceManager {
    pub fn get_resource(&self, id: &str) -> Option<Rc<FluentResource>> {
        self.inner
            .resources
            .iter()
            .find(|r| r.0 == id)
            .map(|(_, res)| res.clone())
    }
}

#[derive(Clone)]
pub enum Bundles<S>
where
    S: StreamProvider,
{
    Iter(Rc<Cache<S::BundlesIter, Rc<FluentResource>>>),
    Stream(Rc<AsyncCache<S::BundlesStream, Rc<FluentResource>>>),
}

pub struct Localization2<S>
where
    S: StreamProvider,
{
    bundles: OnceCell<Bundles<S>>,
    locales: Vec<LanguageIdentifier>,
    provider: S,
    res_ids: Vec<String>,
}

impl<S> Localization2<S>
where
    S: StreamProvider,
{
    pub fn new(res_ids: Vec<String>, provider: S) -> Self {
        let locales = vec!["en".parse().unwrap()];
        Self {
            bundles: OnceCell::new(),
            provider,
            locales,
            res_ids,
        }
    }

    pub fn add_resource_id(&mut self, res_id: String) {
        self.res_ids.push(res_id);
        self.bundles.take();
    }

    pub fn remove_resource_id(&mut self, res_id: &str) {
        self.res_ids.retain(|res| res != res_id);
        self.bundles.take();
    }

    pub fn format_value<'l>(&self, _id: &str) -> impl Future<Output = Option<Cow<'l, str>>>
    where
        <S as StreamProvider>::BundlesStream: Unpin,
    {
        let bundles: &Bundles<_> = self.get_bundles();
        let bundles = if let Bundles::Stream(stream) = bundles {
            stream.clone()
        } else {
            panic!();
        };
        async move {
            use futures::StreamExt;
            let mut bundles = bundles.stream();
            while let Some(_bundle) = bundles.next().await {
                println!("Next bundle!");
            }
            None
            // match bundles {
            //     Bundles::Iter(_) => { None },
            //     Bundles::Stream(_) => {
            //         // let mut bundles = stream.stream();
            //         // while let Some(_bundle) = bundles.next().await {
            //         //     println!("Next bundle!");
            //         // }
            //         None
            //     }
            // }
        }
    }

    fn get_bundles(&self) -> &Bundles<S> {
        self.bundles.get_or_init(|| {
            Bundles::Stream(Rc::new(AsyncCache::new(self.provider.bundles_stream(
                self.locales.clone().into_iter(),
                self.res_ids.clone(),
            ))))
        })
    }
}

pub trait StreamProvider {
    type BundlesStream: Stream<Item = FluentBundleResult<Rc<FluentResource>>> + Clone;
    type BundlesIter: Iterator<Item = FluentBundleResult<Rc<FluentResource>>> + Clone;

    fn bundles_stream(
        &self,
        locales: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
        res_ids: Vec<String>,
    ) -> Self::BundlesStream;
}

impl StreamProvider for ResourceManager {
    type BundlesStream = BundleStream;
    type BundlesIter = BundleIter;

    fn bundles_stream(
        &self,
        locales: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
        res_ids: Vec<String>,
    ) -> Self::BundlesStream {
        BundleStream {
            res_ids,
            locales,
            res_mgr: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct BundleStream {
    res_ids: Vec<String>,
    locales: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
    res_mgr: ResourceManager,
}

impl Stream for BundleStream {
    type Item = FluentBundleResult<Rc<FluentResource>>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        println!("res_ids: {:?}", self.res_ids);
        if let Some(locale) = self.locales.next() {
            let mut bundle: FluentBundle<Rc<FluentResource>> = FluentBundle::new(vec![locale]);
            for id in &self.res_ids {
                let res = self.res_mgr.get_resource(id).unwrap();
                bundle.add_resource(res).unwrap();
            }
            Some(Ok(bundle)).into()
        } else {
            None.into()
        }
    }
}

#[derive(Clone)]
pub struct BundleIter {
    res_ids: Vec<String>,
    locales: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
    res_mgr: ResourceManager,
}

impl Iterator for BundleIter {
    type Item = FluentBundleResult<Rc<FluentResource>>;

    fn next(&mut self) -> Option<Self::Item> {
        let locale = self.locales.next()?;

        let mut bundle: FluentBundle<Rc<FluentResource>> = FluentBundle::new(vec![locale]);
        for id in &self.res_ids {
            let res = self.res_mgr.get_resource(id).unwrap();
            bundle.add_resource(res).unwrap();
        }
        Some(Ok(bundle))
    }
}
