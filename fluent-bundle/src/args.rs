use crate::types::FluentValue;

/// A map of arguments passed from the code to
/// the localization to be used for message
/// formatting.
pub struct FluentArgs<S>(Vec<(S, FluentValue<S>)>);

impl<S> FluentArgs<S> {
    pub fn get(&self, key: &str) -> Option<&FluentValue<S>>
    where
        S: AsRef<str>,
    {
        for (k, v) in &self.0 {
            if k.as_ref() == key {
                return Some(v);
            }
        }
        None
    }
}

impl<S, V> From<Vec<(S, V)>> for FluentArgs<S>
where
    V: Into<FluentValue<S>>,
{
    fn from(input: Vec<(S, V)>) -> Self {
        Self(input.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}
