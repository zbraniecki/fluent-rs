use crate::cache::{AsyncCache, Cache};
use crate::env::LocalesProvider;
use crate::generator::BundleGenerator;
use once_cell::sync::OnceCell;
use std::borrow::Cow;
use std::future::Future;
use std::rc::Rc;

pub enum Bundles<G>
where
    G: BundleGenerator,
{
    Iter(Rc<Cache<G::Iter, G::Resource>>),
    Stream(Rc<AsyncCache<G::Stream, G::Resource>>),
}

impl<S> Clone for Bundles<S>
where
    S: BundleGenerator,
{
    fn clone(&self) -> Self {
        match self {
            Bundles::Iter(i) => Self::Iter(i.clone()),
            Bundles::Stream(s) => Self::Stream(s.clone()),
        }
    }
}

pub struct Localization2<G, P>
where
    G: BundleGenerator<LocalesIter = P::Iter>,
    P: LocalesProvider,
{
    bundles: OnceCell<Bundles<G>>,
    generator: G,
    provider: P,
    res_ids: Vec<String>,
    sync: bool,
}

impl<G, P> Localization2<G, P>
where
    G: BundleGenerator<LocalesIter = P::Iter>,
    P: LocalesProvider,
{
    pub fn with_env(res_ids: Vec<String>, sync: bool, provider: P, generator: G) -> Self {
        Self {
            bundles: OnceCell::new(),
            generator,
            provider,
            res_ids,
            sync,
        }
    }

    pub fn add_resource_id(&mut self, res_id: String) {
        self.res_ids.push(res_id);
        self.bundles.take();
    }

    pub fn remove_resource_id(&mut self, res_id: String) {
        self.res_ids.retain(|res| res != &res_id);
        self.bundles.take();
    }

    pub fn format_value<'l>(
        &self,
        _id: &str,
        _args: Option<()>,
        _errors: &mut Vec<()>,
    ) -> impl Future<Output = Option<Cow<'l, str>>>
    where
        <G as BundleGenerator>::Stream: Unpin,
    {
        let bundles = self.get_bundles().clone();
        async move {
            use futures::StreamExt;
            match bundles {
                Bundles::Iter(iter) => {
                    let mut iter = iter.into_iter();
                    while let Some(_bundle) = iter.next() {
                        println!("Next bundle sync!");
                    }
                    None
                }
                Bundles::Stream(stream) => {
                    let mut bundles = stream.stream();
                    while let Some(_bundle) = bundles.next().await {
                        println!("Next bundle async!");
                    }
                    None
                }
            }
        }
    }

    fn get_bundles(&self) -> &Bundles<G> {
        self.bundles.get_or_init(|| {
            if self.sync {
                Bundles::Iter(Rc::new(Cache::new(
                    self.generator
                        .bundles_iter(self.provider.locales(), self.res_ids.clone()),
                )))
            } else {
                Bundles::Stream(Rc::new(AsyncCache::new(
                    self.generator
                        .bundles_stream(self.provider.locales(), self.res_ids.clone()),
                )))
            }
        })
    }
}
