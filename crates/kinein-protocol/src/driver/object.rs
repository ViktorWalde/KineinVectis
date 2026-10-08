//! Object-only decoding: serde structs otherwise also accept positional arrays.

use std::{collections::BTreeMap, fmt, marker::PhantomData};

use serde::de::{MapAccess, Visitor, value::MapAccessDeserializer};
use serde::{Deserialize, Deserializer};

/// Decodes a JSON object without losing duplicate-field checks in the typed decoder.
/// Positional arrays cannot stand in for named protocol fields.
pub fn deserialize_object<'de, T: Deserialize<'de>, D: Deserializer<'de>>(
    decoder: D,
) -> Result<T, D::Error> {
    struct ObjectVisitor<T>(PhantomData<T>);

    impl<'de, T: Deserialize<'de>> Visitor<'de> for ObjectVisitor<T> {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a protocol object")
        }

        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<T, A::Error> {
            T::deserialize(MapAccessDeserializer::new(map))
        }
    }

    decoder.deserialize_map(ObjectVisitor(PhantomData))
}

/// Decodes a string-keyed object refusing a repeated key (2026-10-08).
///
/// `BTreeMap`'s own decoder keeps the last value silently; the first one may be
/// what the author reviewed. Typed structs already refuse duplicate fields.
pub fn deserialize_unique_map<'de, V: Deserialize<'de>, D: Deserializer<'de>>(
    decoder: D,
) -> Result<BTreeMap<String, V>, D::Error> {
    struct UniqueVisitor<V>(PhantomData<V>);

    impl<'de, V: Deserialize<'de>> Visitor<'de> for UniqueVisitor<V> {
        type Value = BTreeMap<String, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an object with unique keys")
        }

        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut values = BTreeMap::new();
            while let Some(key) = map.next_key::<String>()? {
                if values.contains_key(&key) {
                    return Err(serde::de::Error::custom(format_args!(
                        "duplicate key `{key}`"
                    )));
                }
                let value = map.next_value()?;
                values.insert(key, value);
            }
            Ok(values)
        }
    }

    decoder.deserialize_map(UniqueVisitor(PhantomData))
}

// Remote derives keep a single field definition. The local wire delegate
// uses their inherent decoder only after the shared helper has required a map.
macro_rules! object_serde {
    ($($kind:ty),+ $(,)?) => {$(
        impl serde::Serialize for $kind {
            fn serialize<S: serde::Serializer>(&self, encoder: S) -> Result<S::Ok, S::Error> {
                <$kind>::serialize(self, encoder)
            }
        }
        impl<'de> serde::Deserialize<'de> for $kind {
            fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
                struct Wire($kind);
                impl<'wire> serde::Deserialize<'wire> for Wire {
                    fn deserialize<D: serde::Deserializer<'wire>>(decoder: D) -> Result<Self, D::Error> {
                        <$kind>::deserialize(decoder).map(Self)
                    }
                }
                $crate::driver::deserialize_object(decoder).map(|wire: Wire| wire.0)
            }
        }
    )+};
}

pub(super) use object_serde;
