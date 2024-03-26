//! Loader of workflow definition resources.

use std::fs;
use std::path::Path;
use std::rc::Rc;

use serde::de::DeserializeOwned;
use url::Url;

use crate::validation::ValidateDef;

/// Loader used through this crate to load workflow definition resources.
///
/// Can load resources from both JSON and YAML[^1] files. Can load resources from file
/// or HTTP(S) URIs.
///
/// [^1]: requires the `yaml` feature (enabled by default).
#[derive(Debug, Default)]
pub struct DefLoader {}

impl DefLoader {
    /// Creates a new default `DefLoader`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads a definition object located at the given URI and returns it.
    ///
    /// If the `validate` feature is enabled, the resource is validated before being returned.
    ///
    /// # Errors
    ///
    /// * [`UnsupportedUriScheme`]: `uri`'s scheme is not supported[^1]
    /// * [`UnsupportedFileFormat`]: `uri`'s file extension is not supported[^2]
    /// * [`UnsupportedOperation`]: operation cannot be performed because of a missing feature
    /// * [`InvalidFileUri`]: `uri` is a `file://` URI but its format is invalid
    /// * [`Io`]: I/O error while loading file content
    /// * [`Json`]: error while deserializing JSON data
    /// * `Yaml`: error while deserializing YAML data[^3]
    /// * `Validation`: definition successfully loaded but determined to be invalid[^4]
    ///
    /// [^1]: currently, only `file://` or `http(s)://` URIs are supported.
    ///
    /// [^2]: currently, only JSON and YAML files are supported. YAML files require
    ///       the `yaml` feature (enabled by default).
    ///
    /// [^3]: requires the `yaml` feature (enabled by default).
    ///
    /// [^4]: requires the `validate` feature (enabled by default).
    ///
    /// [`UnsupportedUriScheme`]: crate::Error::UnsupportedUriScheme
    /// [`UnsupportedFileFormat`]: crate::Error::UnsupportedFileFormat
    /// [`UnsupportedOperation`]: crate::Error::UnsupportedOperation
    /// [`InvalidFileUri`]: crate::Error::InvalidFileUri
    /// [`Io`]: crate::Error::Io
    /// [`Json`]: crate::Error::Json
    pub fn load<T>(&self, uri: &Url) -> crate::Result<Rc<T>>
    where
        T: ValidateDef + DeserializeOwned,
    {
        let bytes = match uri.scheme() {
            "file" => self.load_from_file(uri),
            "http" | "https" => self.load_from_http(uri),
            scheme => Err(crate::Error::UnsupportedUriScheme { scheme: scheme.into() }),
        }?;

        let file_ext = uri
            .path_segments()
            .and_then(|mut p| p.next_back())
            .and_then(|p| Path::new(p).extension())
            .map(|ext| ext.to_ascii_lowercase());
        let file_ext = file_ext
            .as_deref()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        let def = Rc::new(match file_ext {
            "json" => self.load_from_json::<T>(&bytes),
            "yaml" | "yml" => self.load_from_yaml::<T>(&bytes),
            ext => Err(crate::Error::UnsupportedFileFormat { file_ext: ext.into() }),
        }?);

        #[cfg(feature = "validate")]
        {
            def.validate_def()?;
        }

        Ok(def)
    }

    fn load_from_file(&self, uri: &Url) -> crate::Result<Vec<u8>> {
        let path = uri
            .to_file_path()
            .map_err(|_| crate::Error::InvalidFileUri(uri.as_str().into()))?;

        Ok(fs::read(path)?)
    }

    fn load_from_http(&self, _uri: &Url) -> crate::Result<Vec<u8>> {
        unimplemented!("loading resources from HTTP URIs is not currently supported");
    }

    fn load_from_json<T>(&self, bytes: &[u8]) -> crate::Result<T>
    where
        T: DeserializeOwned,
    {
        Ok(serde_json::from_slice(bytes)?)
    }

    fn load_from_yaml<T>(&self, #[allow(unused)] bytes: &[u8]) -> crate::Result<T>
    where
        T: DeserializeOwned,
    {
        #[cfg(feature = "yaml")]
        {
            Ok(serde_yaml::from_slice(bytes)?)
        }

        #[cfg(not(feature = "yaml"))]
        {
            Err(crate::Error::UnsupportedOperation { required_feature: "yaml" })
        }
    }
}
