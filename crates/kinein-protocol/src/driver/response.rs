//! External response envelope with exactly one result or numeric error.

use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Error;
use crate::{JSON_RPC_VERSION, JsonRpcId};

/// A valid response from an adapter, preserving the existing ID type.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Reply<T> {
    /// Successful result; a JSON null is allowed when T supports it.
    Success {
        /// ID copied exactly from the request.
        id: JsonRpcId,
        /// Typed operation result.
        result: T,
    },
    /// Failed request with a numeric error.
    Failure {
        /// Copied ID, or null for an unidentifiable malformed request.
        id: JsonRpcId,
        /// External error, never the textual UI error object.
        error: Error,
    },
}

impl<T> Reply<T> {
    /// Exact identifier for correlation by the owner of the pending request.
    #[must_use]
    pub const fn id(&self) -> &JsonRpcId {
        match self {
            Self::Success { id, .. } | Self::Failure { id, .. } => id,
        }
    }
}

impl<T: Serialize> Serialize for Reply<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut object = serializer.serialize_struct("Reply", 3)?;
        object.serialize_field("jsonrpc", JSON_RPC_VERSION)?;
        object.serialize_field("id", self.id())?;
        match self {
            Self::Success { result, .. } => object.serialize_field("result", result)?,
            Self::Failure { error, .. } => object.serialize_field("error", error)?,
        }
        object.end()
    }
}

// Presence must distinguish a missing result from an explicit JSON null.
#[derive(Default)]
enum Field<T> {
    #[default]
    Missing,
    Present(T),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Field<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer).map(Self::Present)
    }
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
struct Wire<T> {
    jsonrpc: String,
    id: JsonRpcId,
    #[serde(default)]
    result: Field<T>,
    #[serde(default)]
    error: Field<Error>,
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Reply<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;

        let wire: Wire<T> = super::deserialize_object(deserializer)?;
        if wire.jsonrpc != JSON_RPC_VERSION {
            return Err(D::Error::custom("unsupported JSON-RPC version"));
        }
        if !(wire.id.is_null()
            || wire.id.is_string()
            || wire.id.as_i64().is_some()
            || wire.id.as_u64().is_some())
        {
            return Err(D::Error::custom("invalid response identifier"));
        }
        match (wire.result, wire.error) {
            (Field::Present(result), Field::Missing) => Ok(Self::Success {
                id: wire.id,
                result,
            }),
            (Field::Missing, Field::Present(error)) => Ok(Self::Failure { id: wire.id, error }),
            _ => Err(D::Error::custom(
                "response requires exactly one result or error",
            )),
        }
    }
}
