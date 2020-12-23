use crate::generator::{FluentBundleResult, BundleGenerator, BundleIterator, BundleStream};
use fluent_bundle::{FluentBundle, FluentResource};
use fluent_testing::scenarios::structs::FileSource;
use futures::Stream;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Context;
use std::task::Poll;
use unic_langid::LanguageIdentifier;

pub struct BundleIter {
    locales: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
    res_ids: Vec<String>,
    generator: MockGenerator,
}

impl BundleIter {
    fn get_file(&self, locale: &str, res_id: &str) -> Option<String> {
        // for source in self.generator.get_sources().iter() {
        //     if let Some(s) = source.get_file(locale, res_id) {
        //         return Some(s);
        //     }
        // }
        None
    }
}

impl BundleIterator for BundleIter {
    type Resource = FluentResource;
}

impl BundleStream for BundleIter {
    type Resource = FluentResource;
}

impl Iterator for BundleIter {
    type Item = FluentBundleResult<FluentResource>;

    fn next(&mut self) -> Option<Self::Item> {
        let locale = self.locales.next()?;
        let mut bundle = FluentBundle::new(vec![locale.clone()]);

        bundle
            .add_function("PLATFORM", |_positional, _named| "linux".into())
            .expect("Failed to add a function to the bundle.");
        bundle.set_use_isolating(false);

        let locale = locale.to_string();
        let mut errors = vec![];

        for res_id in &self.res_ids {
            if let Some(s) = self.get_file(locale.as_str(), res_id) {
                let res = match FluentResource::try_new(s) {
                    Ok(res) => res,
                    Err((res, err)) => {
                        errors.extend(err.into_iter().map(Into::into));
                        res
                    }
                };
                if let Err(err) = bundle.add_resource(res) {
                    errors.extend(err);
                }
            } else {
                panic!("File {} not available in any source", res_id);
            }
        }
        if errors.is_empty() {
            Some(Ok(bundle))
        } else {
            Some(Err((bundle, errors)))
        }
    }
}

impl Stream for BundleIter {
    type Item = FluentBundleResult<FluentResource>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(locale) = self.locales.next() {
            let mut bundle = FluentBundle::new(vec![locale.clone()]);

            bundle
                .add_function("PLATFORM", |_positional, _named| "linux".into())
                .expect("Failed to add a function to the bundle.");
            bundle.set_use_isolating(false);

            let locale = locale.to_string();
            let mut errors = vec![];

            for res_id in &self.res_ids {
                if let Some(s) = self.get_file(locale.as_str(), res_id) {
                    let res = match FluentResource::try_new(s) {
                        Ok(res) => res,
                        Err((res, err)) => {
                            errors.extend(err.into_iter().map(Into::into));
                            res
                        }
                    };
                    if let Err(err) = bundle.add_resource(res) {
                        errors.extend(err);
                    }
                } else {
                    panic!("File {} not available in any source", res_id);
                }
            }
            if errors.is_empty() {
                Some(Ok(bundle)).into()
            } else {
                Some(Err((bundle, errors))).into()
            }
        } else {
            None.into()
        }
    }
}

#[derive(Default)]
pub struct InnerMockGenerator {
    locales: Vec<LanguageIdentifier>,
    file_sources: Vec<FileSource>,
}

#[derive(Default, Clone)]
pub struct MockGenerator {
    inner: Rc<InnerMockGenerator>,
}

impl MockGenerator {
    pub fn new(locales: Vec<LanguageIdentifier>, file_sources: Vec<FileSource>) -> Self {
        Self {
            inner: Rc::new(InnerMockGenerator {
                locales,
                file_sources,
            }),
        }
    }

    pub fn get_sources(&self) -> &[FileSource] {
        &self.inner.file_sources
    }
}

impl BundleGenerator for MockGenerator {
    type Resource = FluentResource;
    type Iter = BundleIter;
    type Stream = BundleIter;

    fn bundles_iter(&self, res_ids: Vec<String>) -> Self::Iter {
        BundleIter {
            locales: self.inner.locales.clone().into_iter(),
            res_ids,
            generator: self.clone(),
        }
    }

    fn bundles_stream(&self, res_ids: Vec<String>) -> Self::Stream {
        BundleIter {
            locales: self.inner.locales.clone().into_iter(),
            res_ids,
            generator: self.clone(),
        }
    }
}
