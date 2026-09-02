//! `serde` support for [`Ident`], behind the `serde` feature.
//!
//! The wire form of an [`Ident`] is simply its text: it serializes as a
//! string and deserializes from one, interning whatever it reads. That means
//! an `Ident` is interchangeable with a `String` or `&str` in any format that
//! delivers strings as text or as UTF-8 bytes, and
//! a value read back is pointer-equal to [`Ident::new`] of the same text.
//!
//! Some formats also have an *identifier position* — enum variant names and,
//! in self-describing formats such as RON, map keys — where the token is not
//! a quoted string. [`IdentSeed`] reads those, going through
//! [`Deserializer::deserialize_identifier`] rather than `deserialize_str`.
//!
//! The `serde` dependency is optional and does not enable `derive`; every
//! impl here is written by hand so that turning the feature on pulls in
//! no proc-macro crates (`serde_derive`, `syn`, `quote`).
//!
//! [`Deserializer::deserialize_identifier`]: serde::Deserializer::deserialize_identifier

use serde::Deserialize;
use serde::Serialize;

use crate::Ident;

/// Builds an [`Ident`] from whatever text a deserializer hands it.
///
/// The field is the `expecting` message, which differs between the string
/// position and the identifier position.
struct IdentVisitor(&'static str);

impl serde::de::Visitor<'_> for IdentVisitor {
    type Value = Ident;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.0)
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Ident::new(value))
    }

    // `String`'s visitor accepts bytes too, and some binary formats answer
    // `deserialize_str` that way; match it so the two stay interchangeable.
    fn visit_bytes<E: serde::de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
        std::str::from_utf8(value)
            .map(Ident::new)
            .map_err(|_| E::invalid_value(serde::de::Unexpected::Bytes(value), &self))
    }
}

// Serialized as the bare string, so an `Ident` is wire-compatible with the
// `String` or `&str` it was interned from.
impl Serialize for Ident {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Ident {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(IdentVisitor("a string"))
    }
}

/// A [`DeserializeSeed`] that reads an [`Ident`] from an identifier position.
///
/// Requires the `serde` feature.
///
/// [`Ident`]'s ordinary [`Deserialize`] implementation reads string *values*
/// through [`deserialize_str`]. Identifier positions — enum variant names, and
/// map keys in formats that distinguish them — are a separate hook, so this
/// seed goes through [`deserialize_identifier`] instead. Self-describing
/// formats such as RON use that distinction to hand back a bare token like
/// `Forest` rather than a quoted `"Forest"`; formats without the distinction
/// (JSON, for one) forward `deserialize_identifier` to `deserialize_str`, so
/// the seed works there too.
///
/// The intended use is [`EnumAccess::variant_seed`], which lets a
/// hand-written [`Deserialize`] capture a variant name it does not know ahead
/// of time:
///
/// ```
/// # use dpsi::{Ident, IdentSeed};
/// # use serde::de::{Deserializer, EnumAccess, VariantAccess, Visitor};
/// struct Card(Ident);
///
/// impl<'de> serde::Deserialize<'de> for Card {
///     fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
///         struct CardVisitor;
///
///         impl<'de> Visitor<'de> for CardVisitor {
///             type Value = Card;
///
///             fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
///                 f.write_str("a card variant")
///             }
///
///             fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Card, A::Error> {
///                 let (name, variant) = data.variant_seed(IdentSeed)?;
///                 variant.unit_variant()?;
///                 Ok(Card(name))
///             }
///         }
///
///         deserializer.deserialize_enum("Card", &[], CardVisitor)
///     }
/// }
///
/// let card: Card = serde_json::from_str(r#""Forest""#).unwrap();
/// assert_eq!(card.0, Ident::new("Forest"));
/// ```
///
/// [`DeserializeSeed`]: serde::de::DeserializeSeed
/// [`EnumAccess::variant_seed`]: serde::de::EnumAccess::variant_seed
/// [`deserialize_str`]: serde::Deserializer::deserialize_str
/// [`deserialize_identifier`]: serde::Deserializer::deserialize_identifier
pub struct IdentSeed;

impl<'de> serde::de::DeserializeSeed<'de> for IdentSeed {
    type Value = Ident;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(IdentVisitor("an identifier"))
    }
}

#[cfg(test)]
mod tests {
    use serde::de::EnumAccess;
    use serde::de::VariantAccess;
    use serde::de::Visitor;

    use super::IdentSeed;
    use crate::Ident;

    /// A variant name read through [`IdentSeed`] rather than a fixed list.
    #[derive(Debug)]
    struct Variant(Ident);

    impl<'de> serde::Deserialize<'de> for Variant {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct VariantVisitor;

            impl<'de> Visitor<'de> for VariantVisitor {
                type Value = Variant;

                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str("an enum variant")
                }

                fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
                where
                    A: EnumAccess<'de>,
                {
                    let (name, variant) = data.variant_seed(IdentSeed)?;
                    variant.unit_variant()?;
                    Ok(Variant(name))
                }
            }

            // An empty variant list: the seed accepts whatever token arrives.
            deserializer.deserialize_enum("Variant", &[], VariantVisitor)
        }
    }

    #[test]
    fn accepts_utf8_bytes_like_string_does() {
        use serde::Deserialize;
        use serde::de::IntoDeserializer;
        use serde::de::value::BytesDeserializer;
        use serde::de::value::Error;

        let bytes: BytesDeserializer<'_, Error> = b"serde-bytes-path".into_deserializer();
        let ident = Ident::deserialize(bytes).unwrap();
        assert_eq!(ident, Ident::new("serde-bytes-path"));

        let invalid: BytesDeserializer<'_, Error> = b"\xff\xfe".into_deserializer();
        assert!(Ident::deserialize(invalid).is_err());
    }

    #[test]
    fn round_trips_as_a_json_string() {
        let ident = Ident::new("serde-round-trip");

        let json = serde_json::to_string(&ident).unwrap();
        assert_eq!(json, r#""serde-round-trip""#);

        let back: Ident = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ident);
    }

    #[test]
    fn deserialized_value_is_pointer_equal_to_ident_new() {
        let back: Ident = serde_json::from_str(r#""serde-pointer-equal""#).unwrap();

        assert!(std::ptr::eq(
            back.as_str(),
            Ident::new("serde-pointer-equal").as_str()
        ));
    }

    #[test]
    fn ident_seed_reads_a_bare_variant_token() {
        let variant: Variant = serde_json::from_str(r#""Forest""#).unwrap();

        assert_eq!(variant.0, Ident::new("Forest"));
        assert!(std::ptr::eq(
            variant.0.as_str(),
            Ident::new("Forest").as_str()
        ));
    }

    #[test]
    fn deserializing_a_non_string_is_an_error() {
        assert!(serde_json::from_str::<Ident>("42").is_err());
    }
}
