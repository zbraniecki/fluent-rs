use std::borrow::{Borrow, Cow};
use std::future::Future;
use std::marker::PhantomData;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use reiterate::Reiterate;

struct FluentBundleIterator<'loc, R, I, F>
where
    F: Future<Output = FluentBundle<R>>,
    I: Iterator<Item = F> + 'loc,
{
    iter: I,
    phantom: PhantomData<&'loc F>,
}

impl<'loc, R, I, F> Iterator for FluentBundleIterator<'loc, R, I, F>
where
    F: Future<Output = FluentBundle<R>>,
    I: Iterator<Item = F> + 'loc,
{
    type Item = Box<F>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Box::new)
    }
}

pub struct Localization<'loc, R, I, F>
where
    F: Future<Output = FluentBundle<R>>,
    I: Iterator<Item = F> + 'loc,
{
    pub resource_ids: Vec<String>,
    bundles: Reiterate<FluentBundleIterator<'loc, R, I, F>>,
    generate_bundles: Box<dyn FnMut(&[String]) -> FluentBundleIterator<'loc, R, I, F> + 'loc>,
}

impl<'loc, R, I, F> Localization<'loc, R, I, F>
where
    F: Future<Output = FluentBundle<R>>,
    I: Iterator<Item = F>,
{
    pub fn new<G>(resource_ids: Vec<String>, mut generate_bundles: G) -> Self
    where
        G: FnMut(&[String]) -> I + 'loc,
    {
        let mut generate = move |x: &[String]| FluentBundleIterator {
            iter: generate_bundles(x),
            phantom: Default::default(),
        };
        let bundles = Reiterate::new(generate(&resource_ids));
        Localization {
            resource_ids,
            bundles,
            generate_bundles: Box::new(generate),
        }
    }

    pub fn on_change(&mut self) {
        self.bundles = Reiterate::new((self.generate_bundles)(&self.resource_ids));
    }

    pub async fn format_value<'l>(
        &'l self,
        id: &'l str,
        args: Option<&'l FluentArgs<'_>>,
    ) -> Cow<'l, str>
    where
        R: Borrow<FluentResource>,
    {
        for bundle in &self.bundles {
            let bundle = &(*bundle).await;
            // if let Some(msg) = bundle.get_message(id) {
            //     if let Some(pattern) = msg.value {
            //         let mut errors = vec![];
            //         return bundle.format_pattern(pattern, args, &mut errors);
            //     }
            // }
        }
        id.into()
    }
}
