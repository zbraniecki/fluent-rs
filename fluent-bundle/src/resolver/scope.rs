use crate::args::FluentArgs;
use crate::bundle::FluentBundle;
use crate::resource::FluentResource;
use std::borrow::Borrow;

/// State for a single `ResolveValue::to_value` call.
pub struct Scope<'bundle, R, M, A> {
    /// The current `FluentBundleBase` instance.
    pub bundle: &'bundle FluentBundle<R, M>,
    /// The current arguments passed by the developer.
    pub args: Option<FluentArgs<A>>,
}

impl<'bundle, R: Borrow<FluentResource>, M, A> Scope<'bundle, R, M, A> {
    pub fn new(bundle: &'bundle FluentBundle<R, M>, args: Option<FluentArgs<A>>) -> Self {
        Scope { bundle, args }
    }
}
