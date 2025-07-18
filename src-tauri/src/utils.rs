use std::{any::Any, env::current_dir, fs::create_dir_all, path::PathBuf};

use color_eyre::eyre::eyre;
use futures_util::TryFutureExt;
use kameo::{Actor, actor::RemoteActorRef, prelude::Message, remote::RemoteMessage};
use libp2p::PeerId;
use serde::Serialize;
use sqlx::{QueryBuilder, Sqlite};
use tokio::sync::oneshot;
use tracing::warn;
use uuid::Uuid;

use crate::{
    actors::{
        Askable,
        gateway::{GATEWAY_ACTOR, GatewayActor},
    },
    error::{AppError, Result},
    keys::Signed,
};

const BUNDLE_IDENTIFIER: &str = "app.evo-design.com";

/// Path to the config directory for the application.
/// Falls back to the current directory if the config directory cannot be determined.
pub fn get_config_dir() -> PathBuf {
    let mut path = match dirs::config_dir() {
        Some(dir) => dir,
        None => {
            warn!("Could not determine config directory. Attempting to use current directory.");
            current_dir().unwrap()
        }
    };
    path.push(BUNDLE_IDENTIFIER);
    if !path.exists() {
        create_dir_all(&path).unwrap();
    }
    path
}

/// Path to the data directory for the application.
/// Falls back to the current directory if the data directory cannot be determined.
pub fn get_data_dir() -> PathBuf {
    let mut path = match dirs::data_dir() {
        Some(dir) => dir,
        None => {
            warn!("Could not determine config directory. Attempting to use current directory.");
            current_dir().unwrap()
        }
    };
    path.push(BUNDLE_IDENTIFIER);
    if !path.exists() {
        create_dir_all(&path).unwrap();
    }
    path
}

/// Path to the models directory for the application.
/// Falls back to the current directory if the models directory cannot be determined.
pub fn get_models_dir() -> PathBuf {
    let mut path = get_data_dir();
    path.push("models");
    if !path.exists() {
        create_dir_all(&path).unwrap();
    }
    path
}

/// Path to the workflow directory for the application.
/// Falls back to the current directory if the workflow directory cannot be determined.
pub fn get_workflow_dir() -> PathBuf {
    let mut path = get_data_dir();
    path.push("workflow");
    if !path.exists() {
        create_dir_all(&path).unwrap();
    }
    path
}

/// Returns a gateway ID for the given peer ID.
/// Uses a random Gateway ID that should exist in their registry
pub fn get_gateway_id(peer_id: &PeerId) -> String {
    format!("gateway-{peer_id}")
}

/// Sends a message to a remote actor and awaits the reply.
/// This works simliarly to `ask` but instead of sending a message to a local actor,
/// it sends a message to a remote actor by using the `GatewayActor` as a proxy and working around
/// the limitations of `ask`.
pub async fn tell_ask<T, A>(
    actor: &RemoteActorRef<GatewayActor>,
    msg: T,
) -> Result<<A as Askable<T>>::ActualReply>
where
    GatewayActor: RemoteMessage<Signed<T>>
        + Message<Signed<T>>
        + RemoteMessage<Signed<<A as Askable<T>>::ActualReply>>
        + Message<Signed<<A as Askable<T>>::ActualReply>>,
    T: Send + Sync + Serialize + 'static,
    A: Askable<T>,
{
    let (tx, rx) = oneshot::channel::<Box<dyn Any + Send + Sync + 'static>>();
    let signed = Signed::new(msg);
    tokio::try_join!(
        GATEWAY_ACTOR
            .get()
            .unwrap()
            .ask(SaveTask {
                sender: tx,
                task_id: Uuid::new_v4(),
            })
            .send()
            .map_err(|e| AppError::SendError(e.to_string())),
        actor.tell(&signed).send().map_err(|e| e.into())
    )?;
    let reply = rx
        .await
        .map_err(|e| eyre!("Error receiving reply from gateway: {e}"))?;
    match reply.downcast::<Result<<A as Askable<T>>::ActualReply>>() {
        Ok(r) => *r,
        Err(_) => Err(eyre!("Invalid reply type received from actor").into()),
    }
}

pub struct SaveTask {
    pub sender: oneshot::Sender<Box<dyn Any + Send + Sync + 'static>>,
    pub task_id: Uuid,
}

pub fn add_where() -> impl FnMut(&mut QueryBuilder<Sqlite>) {
    let mut first_condition = true;
    move |qb: &mut QueryBuilder<Sqlite>| {
        if first_condition {
            qb.push(" WHERE ");
            first_condition = false;
        } else {
            qb.push(" AND ");
        }
    }
}
