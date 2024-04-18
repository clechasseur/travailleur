//! Types and traits pertaining to workflow definition validation.

use crate::detail::GardeValidate;

/// Trait used for workflow definition validation.
///
/// This trait is implemented for all workflow definition types, regardless of
/// whether the `validate` feature is enabled or not. If the `validate` feature
/// is disabled, trying to performm validation will result in an [`Error::FeatureDisabled`].
///
/// [`Error::FeatureDisabled`]: crate::Error::FeatureDisabled
pub trait ValidateDefinition: GardeValidate {
    #[cfg_attr(
        feature = "validate",
        doc = r"
            Validates this definition object.

            Effectively delegates to [`garde::Validate::validate`].

            # Errors

            * [`ValidationFailed`](crate::Error::ValidationFailed): There were validation errors.
        "
    )]
    #[cfg_attr(
        not(feature = "validate"),
        doc = r"
            Validates this definition object.

            Always returns [`FeatureDisabled`] because the `validate` feature is disabled.

            [`FeatureDisabled`]: crate::Error::FeatureDisabled
        "
    )]
    fn validate_definition(&self) -> crate::Result<()> {
        #[cfg(feature = "validate")]
        {
            self.validate(&()).map_err(|err| err.into())
        }

        #[cfg(not(feature = "validate"))]
        {
            Err(crate::Error::FeatureDisabled { required_feature: "validate" })
        }
    }
}

impl<T> ValidateDefinition for T where T: GardeValidate {}
