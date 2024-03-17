// TODO re-enable if needed, otherwise delete
// macro_rules! define_schema_newtype_impl {
//     (Index, $typ:ty, $inner_typ:ty) => {
//         impl<Idx> ::std::ops::Index<Idx> for $typ
//         where
//             $inner_typ : ::std::ops::Index<Idx>,
//         {
//             type Output = <$inner_typ as ::std::ops::Index<Idx>>::Output;
//
//             fn index(&self, index: Idx) -> &Self::Output {
//                 self.0.index(index)
//             }
//         }
//     };
//     (Display, $typ:ty, $_inner_typ:ty) => {
//         impl ::std::fmt::Display for $typ {
//             fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
//                 write!(f, "{}", self.0)
//             }
//         }
//     };
//     (FromStr, $typ:ty, $inner_typ:ty) => {
//         impl ::std::str::FromStr for $typ {
//             type Err = <$inner_typ as ::std::str::FromStr>::Err;
//
//             fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
//                 Ok(Self(<$inner_typ>::from_str(s)?))
//             }
//         }
//     };
//     (PartialEq, $typ:ty, $_inner_typ:ty) => {
//         impl ::std::cmp::PartialEq for $typ {
//             fn eq(&self, other: &Self) -> bool {
//                 self.0.eq(&other.0)
//             }
//         }
//     };
//     (Eq, $typ:ty, $inner_typ:ty) => {
//         crate::detail::newtype::define_schema_newtype_impl!(PartialEq, $typ, $inner_typ);
//
//         impl ::std::cmp::Eq for $typ {}
//     };
//     (PartialOrd, $typ:ty, $_inner_typ:ty) => {
//         impl ::std::cmp::PartialOrd for $typ {
//             fn partial_cmp(&self, other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
//                 self.0.partial_cmp(&other.0)
//             }
//         }
//     };
//     (Ord, $typ:ty, $_inner_typ:ty) => {
//         impl ::std::cmp::PartialOrd for $typ {
//             fn partial_cmp(&self, other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
//                 Some(self.0.cmp(&other.0))
//             }
//         }
//
//         impl ::std::cmp::Ord for $typ {
//             fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
//                 self.0.cmp(&other.0)
//             }
//         }
//     };
// }
//
// macro_rules! define_schema_newtype {
//     (
//         $(#[$attr:meta])*
//         $vis:vis struct $typ:ident(
//             $(#[$inner_attr:meta])*
//             $inner_vis:vis String
//         );
//     ) => {
//         crate::detail::newtype::define_schema_newtype! {
//             #[derive(Debug, Clone)]
//             $(#[$attr])*
//             $vis struct $typ[Display, FromStr, Eq, Ord](
//                 $(#[$inner_attr])*
//                 $inner_vis String
//             );
//         }
//     };
//     (
//         $(#[$attr:meta])*
//         $vis:vis struct $typ:ident(
//             $(#[$inner_attr:meta])*
//             $inner_vis:vis Vec<String>
//         );
//     ) => {
//         crate::detail::newtype::define_schema_newtype! {
//             #[derive(Debug, Clone)]
//             $(#[$attr])*
//             $vis struct $typ[Index, Eq, Ord](
//                 $(#[$inner_attr])*
//                 $inner_vis Vec<String>
//             );
//         }
//     };
//     (
//         $(#[$attr:meta])*
//         $vis:vis struct $typ:ident[
//             $( $typ_impl:ident ),* $(,)?
//         ](
//             $(#[$inner_attr:meta])*
//             $inner_vis:vis $inner_typ:ty
//         );
//     ) => {
//         #[derive(::serde::Serialize, ::serde::Deserialize)]
//         $(#[$attr])*
//         #[serde(transparent)]
//         $vis struct $typ(
//             $(#[$inner_attr])*
//             $inner_vis $inner_typ
//         );
//
//         impl ::std::ops::Deref for $typ {
//             type Target = $inner_typ;
//
//             fn deref(&self) -> &Self::Target {
//                 &self.0
//             }
//         }
//
//         impl ::std::convert::From<$inner_typ> for $typ {
//             fn from(value: $inner_typ) -> Self {
//                 Self(value)
//             }
//         }
//
//         impl ::std::convert::From<$typ> for $inner_typ {
//             fn from(value: $typ) -> Self {
//                 value.0
//             }
//         }
//
//         $(
//             crate::detail::newtype::define_schema_newtype_impl!($typ_impl, $typ, $inner_typ);
//         )*
//     };
// }
//
// pub(crate) use define_schema_newtype_impl;
// pub(crate) use define_schema_newtype;
