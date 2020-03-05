use super::*;

pub struct IntlLangMemoizer {
    lang: LanguageIdentifier,
    map: type_map::concurrent::TypeMap,
}

impl IntlLangMemoizer {
    pub fn new(lang: LanguageIdentifier) -> Self {
        Self {
            lang,
            map: type_map::concurrent::TypeMap::new(),
        }
    }

    pub fn try_get<T: Memoizable + Sync + Send + 'static>(&mut self, args: T::Args) -> Result<&T, T::Error>
    where
        T::Args: Eq + Sync + Send,
    {
        let cache = self
            .map
            .entry::<HashMap<T::Args, T>>()
            .or_insert_with(HashMap::new);

        let e = match cache.entry(args.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let val = T::construct(self.lang.clone(), args)?;
                entry.insert(val)
            }
        };
        Ok(e)
    }
}
