//! Cache for resources referred to by workflow definitions.

use std::any::{type_name, type_name_of_val, Any};
use std::collections::HashMap;
use std::rc::Rc;

use serde::de::DeserializeOwned;
use url::Url;

use crate::detail::IntoOpt;
use crate::loader::DefLoader;
use crate::validation::ValidateDef;

/// Cache for resources referred to by workflow definitions, including sub-workflow definitions, etc.
///
/// The first time a resource is accessed, it is loaded using a [`DefLoader`]. Resources are then
/// cached by URI, so they can be fetched quickly if reused multiple times in a workflow definition.
///
/// # Thread-safety
///
/// **This class is not thread-safe**. Resources are cached in [`Rc`]s, so they cannot be
/// shared between threads/tasks. Each thread/task should have its own `Loader`.
#[derive(Debug, Default)]
pub struct DefCache {
    loader: DefLoader,
    cache: HashMap<Url, Rc<dyn Any>>,
}

impl DefCache {
    /// Creates a new empty `DefCache`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fetches a definition object from the cache, loading it on the first call.
    ///
    /// * If the cache already contains a definition object for the given URI, it is returned.
    /// * Otherwise, we use a [`DefLoader`] to load the definition object and store it in the cache.
    ///
    /// # Errors
    ///
    /// Any error returned by [`DefLoader::load`], in addition to:
    ///
    /// * [`Url`]: An invalid URI was passed
    /// * [`InvalidDowncast`]: caller asked for a definition object of type `T` but an existing
    ///                        object of a different type was found in cache
    ///
    /// [`Url`]: crate::Error::Url
    /// [`InvalidDowncast`]: crate::Error::InvalidDowncast
    pub fn get_or_insert<T, U>(&mut self, uri: U) -> crate::Result<Rc<T>>
    where
        T: ValidateDef + DeserializeOwned + Any,
        U: TryInto<Url>,
        <U as TryInto<Url>>::Error: IntoOpt<crate::Error>,
    {
        let uri = uri.try_into().map_err(|err| {
            err.into_opt()
                .expect("if try_info fails, an error should be returned")
        })?;

        if let Some(def) = self.cache.get(&uri) {
            return Rc::clone(def).downcast::<T>().map_err(|actual| {
                crate::Error::InvalidDowncast {
                    expected_type: type_name::<T>(),
                    actual_type: type_name_of_val(&*actual),
                }
            });
        }

        let def = self.loader.load(&uri)?;
        self.cache.insert(uri, Rc::clone(&def) as Rc<dyn Any>);

        Ok(def)
    }
}
