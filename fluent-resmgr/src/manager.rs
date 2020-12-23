use crate::errors::ResourceManagerError;
use crate::fetcher::FSFileFetcher;
use fluent_bundle::{FluentBundle, FluentResource};
use fluent_fallback::generator::BundleGenerator;
use fluent_fallback::generator::{BundleIterator, BundleStream};
use fluent_fallback::Localization;
use fluent_io::AsyncCache;
use fluent_io::FileFetcher;
use futures::ready;
use futures::stream::Collect;
use futures::stream::FuturesOrdered;
use futures::stream::Stream;
use futures::stream::StreamExt;
use futures::FutureExt;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use unic_langid::LanguageIdentifier;

fn create_resource<E: ErrorReporter>(source: String, reporter: &E) -> FluentResource {
    match FluentResource::try_new(source) {
        Ok(res) => res,
        Err((res, err)) => {
            reporter.report_errors(err);
            res
        }
    }
}

async fn read_resource<P>(path: String, manager: FluentResourceManager<P>) -> FluentResourceOption
where
    P: ErrorReporter,
{
    manager
        .fetcher
        .fetch(&path)
        .await
        .ok()
        .map(|source| Rc::new(create_resource(source, &manager.inner.provider)))
}

type FluentResourceOption = Option<Rc<FluentResource>>;
type ResourceStatus = fluent_io::ValueStatus<FluentResourceOption>;

pub type ResourceSetStream = Collect<FuturesOrdered<ResourceStatus>, Vec<FluentResourceOption>>;

pub trait LocalesProvider {
    fn locales(&self) -> <Vec<LanguageIdentifier> as IntoIterator>::IntoIter;
}

pub trait ErrorReporter {
    fn report_errors<E: std::error::Error>(&self, errors: Vec<E>);
}

struct InnerResourceManager<P> {
    resources: RefCell<AsyncCache<FluentResourceOption>>,
    path_scheme: String,
    provider: P,
}

#[derive(Clone)]
pub struct FluentResourceManager<P> {
    inner: Rc<InnerResourceManager<P>>,
    fetcher: FSFileFetcher,
}

impl<P> FluentResourceManager<P>
where
    P: LocalesProvider + ErrorReporter + Clone + 'static,
{
    pub fn with_provider(path_scheme: String, provider: P) -> Self {
        Self {
            inner: Rc::new(InnerResourceManager {
                resources: RefCell::new(AsyncCache::new()),
                path_scheme,
                provider,
            }),
            fetcher: FSFileFetcher,
        }
    }

    fn get_path(&self, locale: &LanguageIdentifier, path: &str) -> String {
        self.inner
            .path_scheme
            .replace("{locale}", &locale.to_string())
            .replace("{res_id}", path)
    }

    fn get_resource_sync(&self, path: &str, locale: &LanguageIdentifier) -> FluentResourceOption {
        let full_path = self.get_path(locale, path);
        self.inner
            .resources
            .borrow_mut()
            .get_or_insert_with_sync(&full_path, || {
                if let Ok(source) = self.fetcher.fetch_sync(&full_path) {
                    let result = create_resource(source, &self.inner.provider);
                    Some(Rc::new(result))
                } else {
                    None
                }
            })
    }

    fn get_resource(&self, path: &str, locale: &LanguageIdentifier) -> ResourceStatus {
        let full_path = self.get_path(locale, path);
        self.inner
            .resources
            .borrow_mut()
            .get_or_insert_with(&full_path, || {
                ResourceStatus::Loading(
                    read_resource(full_path.clone(), (*self).clone())
                        .boxed_local()
                        .shared(),
                )
            })
    }

    pub fn create_localization(&self, res_ids: Vec<String>, sync: bool) -> Localization<Self> {
        Localization::with_generator(res_ids, sync, self.clone())
    }
}

pub struct BundleIter<P> {
    locales: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
    res_ids: Vec<String>,
    generator: FluentResourceManager<P>,
}

impl<P> BundleIterator for BundleIter<P>
where
    P: LocalesProvider + ErrorReporter + Clone + 'static,
{
    type Resource = Rc<FluentResource>;
}

impl<P> Iterator for BundleIter<P>
where
    P: LocalesProvider + ErrorReporter + Clone + 'static,
{
    type Item = FluentBundle<Rc<FluentResource>>;

    fn next(&mut self) -> Option<Self::Item> {
        let locale = self.locales.next()?;
        let mut bundle = FluentBundle::new(vec![locale.clone()]);

        for res_id in &self.res_ids {
            if let Some(res) = self.generator.get_resource_sync(res_id, &locale) {
                if let Some(errors) = bundle.add_resource(res).err() {
                    self.generator.inner.provider.report_errors(errors);
                }
            } else {
                self.generator
                    .inner
                    .provider
                    .report_errors(vec![ResourceManagerError::MissingResource(res_id.clone())]);
                return None;
            }
        }
        Some(bundle)
    }
}

pub struct BundleStr<P> {
    locales: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
    res_ids: Vec<String>,
    generator: FluentResourceManager<P>,
    current_stream: Option<(LanguageIdentifier, ResourceSetStream)>,
}

impl<P> BundleStream for BundleStr<P>
where
    P: LocalesProvider + ErrorReporter + Clone + 'static,
{
    type Resource = Rc<FluentResource>;
}

impl<P> Stream for BundleStr<P>
where
    P: LocalesProvider + ErrorReporter + Clone + 'static,
{
    type Item = FluentBundle<Rc<FluentResource>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some((_, stream)) = &mut self.current_stream {
                let set = ready!(stream.poll_unpin(cx));
                let (locale, _) = self.current_stream.take().unwrap();
                let mut bundle = FluentBundle::new(vec![locale.clone()]);
                for (res_idx, res) in set.into_iter().enumerate() {
                    if let Some(res) = res {
                        if let Some(errors) = bundle.add_resource(res).err() {
                            self.generator.inner.provider.report_errors(errors);
                        }
                    } else {
                        let res_id = &self.res_ids[res_idx];
                        self.generator.inner.provider.report_errors(vec![
                            ResourceManagerError::MissingResource(res_id.to_string()),
                        ]);
                        break;
                    }
                }
                return Some(bundle).into();
            } else if let Some(locale) = self.locales.next() {
                let mut futures = vec![];
                for res_id in &self.res_ids {
                    futures.push(self.generator.get_resource(res_id, &locale));
                }
                self.current_stream = Some((
                    locale,
                    futures.into_iter().collect::<FuturesOrdered<_>>().collect(),
                ));
            } else {
                break;
            }
        }
        None.into()
    }
}

impl<P> BundleGenerator for FluentResourceManager<P>
where
    P: LocalesProvider + ErrorReporter + Clone + 'static,
{
    type Resource = FluentResource;
    type Iter = BundleIter<P>;
    type Stream = BundleStr<P>;

    fn bundles_iter(&self, res_ids: Vec<String>) -> Self::Iter {
        BundleIter {
            locales: self.inner.provider.locales(),
            res_ids,
            generator: self.to_owned(),
        }
    }

    fn bundles_stream(&self, res_ids: Vec<String>) -> Self::Stream {
        BundleStr {
            locales: self.inner.provider.locales(),
            res_ids,
            generator: self.clone(),
            current_stream: None,
        }
    }
}
