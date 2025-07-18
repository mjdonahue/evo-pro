use std::sync::{Arc, LazyLock, OnceLock, RwLock};

use kameo::remote::Keypair;
use libp2p::{PeerId, identity::PublicKey};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

use crate::{error::Result, utils::get_data_dir};

pub static KEY_PAIR: LazyLock<Arc<RwLock<Keypair>>> =
    LazyLock::new(|| Arc::new(RwLock::new(fetch_peer_keypair())));
pub static PEER_ID: OnceLock<PeerId> = OnceLock::new();

pub fn fetch_user_keypair() -> Keypair {
    let key_path = get_data_dir().join("keypair.proto");
    if key_path.is_file() {
        Keypair::from_protobuf_encoding(&std::fs::read(key_path).expect("failed to read keypair"))
            .expect("failed to decode keypair")
    } else {
        let pair = Keypair::generate_ed25519();
        std::fs::write(
            key_path,
            pair.to_protobuf_encoding()
                .expect("keypair should have valid protobuf encoding"),
        )
        .expect("failed to write keypair");
        pair
    }
}

pub fn fetch_peer_keypair() -> Keypair {
    let key_path = get_data_dir().join("peer-keypair.proto");
    if key_path.is_file() {
        Keypair::from_protobuf_encoding(&std::fs::read(key_path).expect("failed to read keypair"))
            .expect("failed to decode keypair")
    } else {
        let pair = Keypair::generate_ed25519();
        std::fs::write(
            key_path,
            pair.to_protobuf_encoding()
                .expect("keypair should have valid protobuf encoding"),
        )
        .expect("failed to write keypair");
        pair
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PubKeyWrapper(
    #[serde(serialize_with = "serialize_key", deserialize_with = "deserialize_key")] pub PublicKey,
);

#[derive(Clone, Serialize, Deserialize)]
pub struct Signed<T> {
    inner: T,
    signature: Vec<u8>,
    public_key: PubKeyWrapper,
    client_peer_id: PeerId,
    task_id: Option<Uuid>,
}

impl<T> Signed<T> {
    pub fn into_inner(self) -> T {
        self.inner
    }
    pub fn inner(&self) -> &T {
        &self.inner
    }
    pub fn client_peer_id(&self) -> &PeerId {
        &self.client_peer_id
    }
    pub fn task_id(&self) -> Option<&Uuid> {
        self.task_id.as_ref()
    }
    pub fn take_task_id(&mut self) -> Option<Uuid> {
        self.task_id.take()
    }
}

impl<T: Serialize> Signed<T> {
    pub fn new(inner: T) -> Self {
        Self::with_task(inner, None)
    }

    pub fn with_task(inner: T, task_id: Option<Uuid>) -> Self {
        let key_pair = KEY_PAIR.read().unwrap().clone();
        let signature = key_pair.sign(&serde_json::to_vec(&inner).unwrap()).unwrap();
        Self {
            inner,
            signature,
            public_key: PubKeyWrapper(key_pair.public()),
            client_peer_id: PEER_ID.get().cloned().unwrap(),
            task_id,
        }
    }

    pub fn verify_signature(&self) -> bool {
        self.public_key
            .0
            .verify(&serde_json::to_vec(&self.inner).unwrap(), &self.signature)
    }
}

fn serialize_key<S>(value: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_bytes(&value.encode_protobuf())
}

fn deserialize_key<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
    PublicKey::try_decode_protobuf(&bytes).map_err(serde::de::Error::custom)
}
