use std::fmt::Display;
use std::hash::Hash;
use std::ops::Rem;
use std::str::FromStr;

use itertools::Itertools;
use num::Zero;

use crate::workflow::definition::common::ValidatedNonNegativeNumber;

// TODO re-enable if needed, otherwise delete
// macro_rules! garde_append {
//     ($path:ident, $report:ident, $err:expr) => {
//         $report.append($path(), ::garde::Error::new($err));
//     };
//     ($parent:ident, $field:expr, $report:ident, $err:expr) => {{
//         let mut path = ::garde::util::nested_path!($parent, $field);
//         crate::detail::garde::garde_append!(path, $report, $err);
//     }};
// }
//
// macro_rules! garde_check_one_is_set {
//     ($either:expr, $either_name:expr, $or:expr, $or_name:expr, $parent:ident, $report:ident) => {
//         if $either.is_none() || $or.is_none() {
//             crate::detail::garde::garde_append!(
//                 $parent,
//                 $report,
//                 format!("one of `{}` or `{}` must be set", $either_name, $or_name)
//             );
//         }
//     };
// }
//
// macro_rules! garde_check_option_not_empty {
//     ($field:expr, $field_name:expr, $parent:ident, $report:ident) => {
//         if $field.as_ref().is_some_and(|val| val.is_empty()) {
//             crate::detail::garde::garde_append!($parent, $field_name, $report, "length must be >= 1");
//         }
//     };
// }
//
// macro_rules! garde_optional_dive {
//     ($field:expr, $field_name:expr, $ctx:ident, $parent:ident, $report:ident) => {
//         {
//             let mut path = ::garde::util::nested_path!($parent, $field_name);
//             $field.validate_into($ctx, &mut path, $report);
//         }
//     };
// }
//
// pub(crate) use {garde_append, garde_check_one_is_set, garde_check_option_not_empty, garde_optional_dive};

pub fn one_must_be_set<'f2, T, U, C>(
    field_name_one: &'static str,
    field_name_two: &'static str,
    field_two: Option<&'f2 U>,
) -> impl FnOnce(&Option<T>, &C) -> garde::Result + 'f2
where
    C: ?Sized,
{
    move |field_one, _ctx| {
        if field_one.is_none() && field_two.is_none() {
            Err(garde::Error::new(format!(
                "at least one of `{}` or `{}` must be set",
                field_name_one, field_name_two
            )))
        } else {
            Ok(())
        }
    }
}

pub fn one_of_three_must_be_set<'f2, 'f3, T, U, V, C>(
    field_name_one: &'static str,
    field_name_two: &'static str,
    field_name_three: &'static str,
    field_two: Option<&'f2 U>,
    field_three: Option<&'f3 V>,
) -> impl FnOnce(&Option<T>, &C) -> garde::Result + 'f2 + 'f3
where
    'f2: 'f3,
    'f3: 'f2,
    C: ?Sized,
{
    move |field_one, _ctx| {
        if field_one.is_none() && field_two.is_none() && field_three.is_none() {
            Err(garde::Error::new(format!(
                "at least one of `{}`, `{}` or `{}` must be set",
                field_name_one, field_name_two, field_name_three
            )))
        } else {
            Ok(())
        }
    }
}

pub fn unique_values<I, C, T>(values: I, _ctx: &C) -> garde::Result
where
    I: IntoIterator<Item = T>,
    T: Eq + Hash + Display,
    C: ?Sized,
{
    match values.into_iter().duplicates().next() {
        Some(dup) => Err(garde::Error::new(format!(
            "values must be unique, but found duplicate item `{}`",
            dup
        ))),
        None => Ok(()),
    }
}

pub fn must_be_a_number<T, S, C>(value: S, _ctx: &C) -> garde::Result
where
    T: FromStr,
    S: AsRef<str>,
    C: ?Sized,
{
    let value = value.as_ref();

    // An empty string is interpreted as 0, so is considered valid.
    if value.is_empty() {
        return Ok(());
    }

    value
        .parse::<T>()
        .map(|_| ())
        .map_err(|_| garde::Error::new(format!("expected a number, found '{}'", value)))
}

pub fn must_be_zero_or_greater<T, C>(value: &T, _ctx: &C) -> garde::Result
where
    T: PartialOrd + Zero + Display,
{
    if *value >= T::zero() {
        Ok(())
    } else {
        Err(garde::Error::new(format!("expected number zero or above, got {}", value)))
    }
}

pub fn must_be<T, C>(expected: T) -> impl FnOnce(&T, &C) -> garde::Result
where
    T: PartialEq + Display,
    C: ?Sized,
{
    move |value, _ctx| {
        (*value == expected)
            .then_some(())
            .ok_or_else(|| garde::Error::new(format!("expected '{}', got '{}'", expected, value)))
    }
}

pub fn must_be_multiple_of<'a, M, T, C>(multiple: M) -> impl FnOnce(&'a T, &C) -> garde::Result
where
    M: Copy + Rem + Zero + Display,
    <M as Rem>::Output: PartialEq<M>,
    T: Display + 'a,
    &'a T: TryInto<ValidatedNonNegativeNumber<M>>,
    <&'a T as TryInto<ValidatedNonNegativeNumber<M>>>::Error: Display,
    C: ?Sized,
{
    move |value, _ctx| match value.try_into() {
        Ok(value) => {
            if value.value() % multiple != M::zero() {
                Err(garde::Error::new(format!(
                    "expected {} to be a multiple of {}",
                    value, multiple
                )))
            } else {
                Ok(())
            }
        },
        Err(err) => {
            Err(garde::Error::new(format!("expected valid number, got '{}': {}", value, err)))
        },
    }
}

pub fn must_be_optional_multiple_of<'a, M, T, C>(
    multiple: M,
) -> impl FnOnce(&'a Option<T>, &C) -> garde::Result
where
    M: Copy + Rem + Zero + Display,
    <M as Rem>::Output: PartialEq<M>,
    T: Display + 'a,
    &'a T: TryInto<ValidatedNonNegativeNumber<M>>,
    <&'a T as TryInto<ValidatedNonNegativeNumber<M>>>::Error: Display,
    C: ?Sized,
{
    move |value, ctx| match value {
        Some(value) => must_be_multiple_of(multiple)(value, ctx),
        None => Ok(()),
    }
}
