//! Identifier newtypes and enums shared by several services, plus the macros used to
//! define service-specific ones consistently.

/// Defines a string-backed identifier newtype.
///
/// The generated type derives `Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord`,
/// (de)serializes transparently as a string (also accepting a JSON integer, for ids that
/// some endpoints send as numbers), and implements `Display`, `FromStr`, `AsRef<str>`,
/// `Borrow<str>`, `From<String>`, `From<&str>` and `From<Self> for String`.
///
/// ```
/// polyoxide_core::string_id! {
///     /// A widget id.
///     pub struct WidgetId;
/// }
///
/// let id = WidgetId::from("w-1");
/// assert_eq!(id.as_str(), "w-1");
/// let parsed: WidgetId = serde_json::from_str("42").unwrap();
/// assert_eq!(parsed, "42");
/// ```
#[macro_export]
macro_rules! string_id {
    ($(#[$meta:meta])* $vis:vis struct $name:ident;) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        $vis struct $name(::std::string::String);

        impl $name {
            /// Creates the identifier from any string-like value.
            #[must_use]
            pub fn new(id: impl ::std::convert::Into<::std::string::String>) -> Self {
                Self(id.into())
            }

            /// The identifier as a string slice.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consumes the identifier, returning the inner string.
            #[must_use]
            pub fn into_inner(self) -> ::std::string::String {
                self.0
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = ::std::convert::Infallible;
            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                Ok(Self(s.to_owned()))
            }
        }

        impl ::std::convert::AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl ::std::borrow::Borrow<str> for $name {
            fn borrow(&self) -> &str {
                &self.0
            }
        }

        impl ::std::convert::From<::std::string::String> for $name {
            fn from(id: ::std::string::String) -> Self {
                Self(id)
            }
        }

        impl ::std::convert::From<&str> for $name {
            fn from(id: &str) -> Self {
                Self(id.to_owned())
            }
        }

        impl ::std::convert::From<&::std::string::String> for $name {
            fn from(id: &::std::string::String) -> Self {
                Self(id.clone())
            }
        }

        impl ::std::convert::From<&$name> for $name {
            fn from(id: &$name) -> Self {
                id.clone()
            }
        }

        impl ::std::convert::From<$name> for ::std::string::String {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl ::std::cmp::PartialEq<str> for $name {
            fn eq(&self, other: &str) -> bool {
                self.0 == other
            }
        }

        impl ::std::cmp::PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.0 == *other
            }
        }

        impl $crate::__private::serde::Serialize for $name {
            fn serialize<S: $crate::__private::serde::Serializer>(
                &self,
                serializer: S,
            ) -> ::std::result::Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> $crate::__private::serde::Deserialize<'de> for $name {
            fn deserialize<D: $crate::__private::serde::Deserializer<'de>>(
                deserializer: D,
            ) -> ::std::result::Result<Self, D::Error> {
                $crate::__private::deserialize_string_id(deserializer).map(Self)
            }
        }
    };
}

/// Defines a `#[non_exhaustive]` string enum with a catch-all `Unknown(String)` variant.
///
/// Each variant maps to its documented wire spelling (optionally with `|`-separated
/// aliases accepted on input). Unrecognised values deserialize to `Unknown(raw)` instead of
/// failing, so new server-side values never break a response. Generated impls: `Debug,
/// Clone, PartialEq, Eq, Hash`, `Display` / `as_str()` (wire spelling), infallible
/// `FromStr`, `From<&str>`, `From<String>`, `Serialize`, `Deserialize`.
///
/// ```
/// polyoxide_core::string_enum! {
///     /// Order side.
///     pub enum Direction {
///         /// Buying.
///         Up => "UP" | "up",
///         /// Selling.
///         Down => "DOWN",
///     }
/// }
///
/// let d: Direction = serde_json::from_str("\"up\"").unwrap();
/// assert_eq!(d, Direction::Up);
/// assert_eq!(d.to_string(), "UP");
/// let other: Direction = serde_json::from_str("\"SIDEWAYS\"").unwrap();
/// assert_eq!(other, Direction::Unknown("SIDEWAYS".to_owned()));
/// ```
#[macro_export]
macro_rules! string_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$vmeta:meta])*
                $variant:ident => $wire:literal $(| $alias:literal)*
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        $vis enum $name {
            $(
                $(#[$vmeta])*
                $variant,
            )+
            /// A value not known to this version of the library, holding the raw wire
            /// string.
            Unknown(::std::string::String),
        }

        impl $name {
            /// The wire spelling of this value.
            #[must_use]
            pub fn as_str(&self) -> &str {
                match self {
                    $( Self::$variant => $wire, )+
                    Self::Unknown(raw) => raw.as_str(),
                }
            }

            /// `true` if this is the `Unknown` catch-all variant.
            #[must_use]
            pub fn is_unknown(&self) -> bool {
                matches!(self, Self::Unknown(_))
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = ::std::convert::Infallible;
            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                Ok(match s {
                    $( $wire $(| $alias)* => Self::$variant, )+
                    other => Self::Unknown(other.to_owned()),
                })
            }
        }

        impl ::std::convert::From<&str> for $name {
            fn from(s: &str) -> Self {
                match s.parse() {
                    Ok(value) => value,
                    Err(never) => match never {},
                }
            }
        }

        impl ::std::convert::From<::std::string::String> for $name {
            fn from(s: ::std::string::String) -> Self {
                Self::from(s.as_str())
            }
        }

        impl $crate::__private::serde::Serialize for $name {
            fn serialize<S: $crate::__private::serde::Serializer>(
                &self,
                serializer: S,
            ) -> ::std::result::Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> $crate::__private::serde::Deserialize<'de> for $name {
            fn deserialize<D: $crate::__private::serde::Deserializer<'de>>(
                deserializer: D,
            ) -> ::std::result::Result<Self, D::Error> {
                let raw = <::std::string::String as $crate::__private::serde::Deserialize>::deserialize(
                    deserializer,
                )?;
                Ok(Self::from(raw.as_str()))
            }
        }
    };
}

string_id! {
    /// A CLOB outcome token id (also called asset id): a decimal-encoded `uint256`, e.g.
    /// `"71321045679252212594626385532706912750332728571942532289631379312455583992563"`.
    pub struct TokenId;
}

string_id! {
    /// An on-chain condition id: `0x` followed by 64 hex characters. The CLOB API calls it
    /// `market`, the Data API `condition`.
    pub struct ConditionId;
}

string_id! {
    /// An EVM address (`0x` followed by 40 hex characters), e.g. a user's proxy wallet.
    ///
    /// The value is kept exactly as given or received; no checksum validation or case
    /// normalisation is applied.
    pub struct Address;
}

string_enum! {
    /// The side of an order or trade.
    pub enum Side {
        /// A buy.
        Buy => "BUY",
        /// A sell.
        Sell => "SELL",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_roundtrip() {
        let id: TokenId = serde_json::from_str("\"123\"").unwrap();
        assert_eq!(id, "123");
        assert_eq!(serde_json::to_string(&id).unwrap(), "\"123\"");
        let numeric: TokenId = serde_json::from_str("123").unwrap();
        assert_eq!(numeric, id);
        assert!(serde_json::from_str::<TokenId>("[1]").is_err());
    }

    #[test]
    fn enums_tolerate_unknown_values() {
        let side: Side = serde_json::from_str("\"BUY\"").unwrap();
        assert_eq!(side, Side::Buy);
        let side: Side = serde_json::from_str("\"\"").unwrap();
        assert_eq!(side, Side::Unknown(String::new()));
        assert!(side.is_unknown());
        assert_eq!(serde_json::to_string(&Side::Sell).unwrap(), "\"SELL\"");
    }
}
