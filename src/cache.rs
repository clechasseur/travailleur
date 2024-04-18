//! Cache for resources referred to by workflow definitions.

use std::any::{type_name, Any};
use std::collections::HashMap;
use std::rc::Rc;

use serde::de::DeserializeOwned;
use url::Url;

use crate::detail::IntoOpt;
use crate::loader::DefinitionLoader;
use crate::validation::ValidateDefinition;

/// Cache for resources referred to by workflow definitions, including sub-workflow definitions, etc.
///
/// The first time a resource is accessed, it is loaded using a [`DefinitionLoader`]. Resources are then
/// cached by URI, so they can be fetched quickly if reused multiple times in a workflow definition.
///
/// # Thread-safety
///
/// **This class is not thread-safe**. Resources are cached in [`Rc`]s, so they cannot be
/// shared between threads/tasks. Each thread/task should have its own [`DefinitionCache`].
#[derive(Debug, Default)]
pub struct DefinitionCache {
    loader: DefinitionLoader,
    cache: HashMap<Url, (Rc<dyn Any>, &'static str)>,
}

impl DefinitionCache {
    /// Creates a new empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fetches a definition object from the cache, loading it on the first call.
    ///
    /// * If the cache already contains a definition object for the given URI, it is returned.
    /// * Otherwise, we use a [`DefinitionLoader`] to load the definition object and store it in the cache.
    ///
    /// # Errors
    ///
    /// Any error returned by [`DefinitionLoader::load`], in addition to:
    ///
    /// * [`InvalidUrl`]: An invalid URI was passed
    /// * [`InvalidCachedObjectType`]: caller asked for a definition object of type `T` but an
    ///                                existing object of a different type was found in cache
    ///
    /// [`InvalidUrl`]: crate::Error::InvalidUrl
    /// [`InvalidCachedObjectType`]: crate::Error::InvalidCachedObjectType
    pub fn get_or_insert<T, U>(&mut self, uri: U) -> crate::Result<Rc<T>>
    where
        T: ValidateDefinition + DeserializeOwned + Any,
        U: TryInto<Url>,
        <U as TryInto<Url>>::Error: IntoOpt<crate::Error>,
    {
        let uri = uri.try_into().map_err(|err| {
            err.into_opt()
                .expect("if try_info fails, an error should be returned")
        })?;

        let def_type_name = type_name::<T>();
        if let Some((def, actual_type)) = self.cache.get(&uri) {
            return Rc::clone(def).downcast::<T>().map_err(|_| {
                crate::Error::InvalidCachedObjectType { expected_type: def_type_name, actual_type }
            });
        }

        let def = self.loader.load(&uri)?;
        self.cache
            .insert(uri, (Rc::clone(&def) as Rc<dyn Any>, def_type_name));

        Ok(def)
    }
}
