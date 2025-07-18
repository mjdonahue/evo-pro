use std::{fmt::Display, ops::Deref, str::FromStr};

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use sqlx::{
    Decode, Encode, Sqlite,
    sqlite::{SqliteTypeInfo, SqliteValueRef},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PeerIdWrapper(pub PeerId);

impl sqlx::Type<Sqlite> for PeerIdWrapper {
    fn type_info() -> SqliteTypeInfo {
        <&[u8] as sqlx::Type<Sqlite>>::type_info()
    }
}

impl Deref for PeerIdWrapper {
    type Target = PeerId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<PeerId> for PeerIdWrapper {
    fn from(peer_id: PeerId) -> Self {
        PeerIdWrapper(peer_id)
    }
}

impl Display for PeerIdWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'r> Decode<'r, Sqlite> for PeerIdWrapper {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let value: Vec<u8> = <Vec<u8> as Decode<Sqlite>>::decode(value)?;
        Ok(PeerIdWrapper(PeerId::from_bytes(&value)?))
    }
}

impl<'q> Encode<'q, Sqlite> for PeerIdWrapper {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let bytes = self.0.to_bytes();
        <Vec<u8> as Encode<Sqlite>>::encode_by_ref(&bytes, buf);
        Ok(sqlx::encode::IsNull::No)
    }
}
