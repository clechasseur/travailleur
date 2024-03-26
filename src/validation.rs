//! Types and traits pertaining to validation.

use crate::detail::GardeValidate;

/// Trait used for workflow definition validation.
pub trait ValidateDef: GardeValidate {
    #[cfg_attr(
        feature = "validate",
        doc = r"
            Validates this definition object.

            Effectively delegates to [`garde::Validate::validate`].

            # Errors

            * [`Validation`](crate::Error::Validation): There were validation errors.
        "
    )]
    #[cfg_attr(
        not(feature = "validate"),
        doc = r"
            Validates this definition object.

            Always returns [`UnsupportedOperation`] because the `validate` feature is disabled.

            [`UnsupportedOperation`]: crate::Error::UnsupportedOperation
        "
    )]
    fn validate_def(&self) -> crate::Result<()> {
        #[cfg(feature = "validate")]
        {
            self.validate(&()).map_err(|err| err.into())
        }

        #[cfg(not(feature = "validate"))]
        {
            Err(crate::Error::UnsupportedOperation { required_feature: "validate" })
        }
    }
}

impl<T> ValidateDef for T where T: GardeValidate {}
