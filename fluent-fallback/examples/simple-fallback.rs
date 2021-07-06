use fluent_bundle::{FluentBundle, FluentResource};
use fluent_fallback::generator::{BundleGenerator, FluentBundleResult};
use fluent_fallback::localization2::*;
use futures::Stream;
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

impl BundleGenerator for ResourceManager {
    type Resource = Rc<FluentResource>;
    type LocalesIter = <Vec<LanguageIdentifier> as IntoIterator>::IntoIter;
    type Stream = BundleStream;
    type Iter = BundleIter;

    fn bundles_stream(
        &self,
        locales: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
        res_ids: Vec<String>,
    ) -> Self::Stream {
        BundleStream {
            res_ids,
            locales,
            res_mgr: self.clone(),
        }
    }

    fn bundles_iter(
        &self,
        locales: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
        res_ids: Vec<String>,
    ) -> Self::Iter {
        BundleIter {
            res_ids,
            locales,
            res_mgr: self.clone(),
        }
    }
}

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
        if let Some(locale) = self.locales.next() {
            println!("res_ids: {:?}", self.res_ids);
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
        println!("res_ids: {:?}", self.res_ids);
        for id in &self.res_ids {
            let res = self.res_mgr.get_resource(id).unwrap();
            bundle.add_resource(res).unwrap();
        }
        Some(Ok(bundle))
    }
}
#[tokio::main]
async fn main() {
    let provider = vec!["en".parse().unwrap()];
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
    let mut errors = vec![];

    let mut loc = Localization2::with_env(res_ids, false, provider, res_mgr);

    let future = loc.format_value("key", None, &mut errors);

    println!("Adding new res_id");
    loc.add_resource_id("menu.ftl".to_string());

    let future2 = loc.format_value("key", None, &mut errors);

    println!("Executing future 1");
    future.await;

    println!("Executing future 2");
    future2.await;

    loc.remove_resource_id("main.ftl".to_string());

    println!("Executing future 3");
    loc.format_value("key", None, &mut errors).await;
}
