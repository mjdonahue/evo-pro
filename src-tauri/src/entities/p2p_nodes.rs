use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::types::Json;
use sqlx::{QueryBuilder, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::entities::PeerIdWrapper;
use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum P2pNodeStatus {
    Online = 0,
    Offline = 1,
    Unreachable = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum P2pNodeType {
    AgentNode = 0,
    GatewayNode = 1,
    RelayNode = 2,
}
#[boilermates("CreateP2pNode")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct P2pNode {
    pub participant_id: Uuid,
    pub peer_id: PeerIdWrapper,
    pub node_type: P2pNodeType, // 0: 'AGENT_NODE', 1: 'GATEWAY_NODE', 2: 'RELAY_NODE'
    pub multiaddr: String,      // libp2p multiaddress
    pub public_key: Option<String>,
    pub capabilities: Option<Json<Value>>, // JSON array of node capabilities
    pub status: P2pNodeStatus,             // 0: 'ONLINE', 1: 'OFFLINE', 2: 'UNREACHABLE'
    #[boilermates(not_in("CreateP2pNode"))]
    pub last_seen: Option<DateTime<Utc>>,
    #[boilermates(not_in("CreateP2pNode"))]
    pub connection_quality: Option<f64>, // 0.0 to 1.0 connection quality score
    #[boilermates(not_in("CreateP2pNode"))]
    pub latency_ms: Option<i64>,
    pub metadata: Option<Json<Value>>, // JSON object with additional metadata
    #[boilermates(not_in("CreateP2pNode"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateP2pNode"))]
    pub updated_at: DateTime<Utc>,
}
/// Additional filtering options for contact queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
pub struct P2pNodeFilter {
    pub node_type: Option<P2pNodeType>,
    pub status: Option<P2pNodeStatus>,
    pub online_only: Option<bool>,
    pub min_connection_quality: Option<f64>,
    pub max_latency_ms: Option<i64>,
    pub has_public_key: Option<bool>,
    pub search_term: Option<String>, // Search in peer_id or multiaddr
    pub last_seen_after: Option<DateTime<Utc>>,
    pub last_seen_before: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new P2P node
    #[instrument(skip(self))]
    pub async fn create_p2p_node(&self, p2p_node: &CreateP2pNode) -> Result<P2pNode> {
        debug!(
            "Creating P2P node with peer_id: {} and participant_id: {}",
            p2p_node.peer_id, p2p_node.participant_id
        );
        let capabilities = p2p_node.capabilities.as_deref();
        let metadata = p2p_node.metadata.as_deref();
        Ok(sqlx::query_as!(
            P2pNode,
            r#"INSERT INTO p2p_nodes (  
                participant_id, peer_id, node_type, multiaddr, public_key,
                capabilities, status, metadata
            ) VALUES (
             ?, ?, ?, ?, ?, ?, ?, ?
             ) RETURNING 
                participant_id AS "participant_id: _", 
                peer_id AS "peer_id: _",
                node_type AS "node_type: P2pNodeType", 
                multiaddr, 
                public_key,
                capabilities AS "capabilities: _", 
                status AS "status: P2pNodeStatus", 
                connection_quality, 
                latency_ms AS "latency_ms: _",
                last_seen AS "last_seen: _", 
                metadata AS "metadata: _", 
                created_at AS "created_at: _", 
                updated_at AS "updated_at: _""#,
            p2p_node.participant_id,
            p2p_node.peer_id,
            p2p_node.node_type,
            p2p_node.multiaddr,
            p2p_node.public_key,
            capabilities,
            p2p_node.status,
            metadata,
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Get P2P node by ID
    #[instrument(skip(self))]
    pub async fn get_p2p_node(
        &self,
        peer_id: &Uuid,
        participant_id: &Uuid,
    ) -> Result<Option<P2pNode>> {
        debug!(
            "Getting P2P node by peer_id: {} and participant_id: {}",
            peer_id, participant_id
        );

        Ok(sqlx::query_as!(
            P2pNode,
            r#"SELECT 
                    participant_id AS "participant_id: _", peer_id AS "peer_id: _", node_type AS "node_type: P2pNodeType", multiaddr AS "multiaddr: _", public_key AS "public_key: _",
                    capabilities AS "capabilities: _", status AS "status: P2pNodeStatus", last_seen AS "last_seen: _", connection_quality AS "connection_quality: _", latency_ms,
                    metadata AS "metadata: _", created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM p2p_nodes WHERE peer_id = ? AND participant_id = ?"#,
            peer_id,
            participant_id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// Get P2P node by peer ID
    #[instrument(skip(self))]
    pub async fn get_p2p_node_by_peer_id(&self, peer_id: &str) -> Result<Option<P2pNode>> {
        debug!("Getting P2P node by peer ID: {}", peer_id);
        debug!("Getting P2P node by peer ID: {:?}", peer_id);

        Ok(sqlx::query_as!(
            P2pNode,
            r#"SELECT 
                    participant_id AS "participant_id: _", peer_id AS "peer_id: _", node_type AS "node_type: P2pNodeType", multiaddr AS "multiaddr: _", public_key AS "public_key: _",
                    capabilities AS "capabilities: _", status AS "status: P2pNodeStatus", last_seen AS "last_seen: _", connection_quality AS "connection_quality: _", latency_ms,
                    metadata AS "metadata: _", created_at AS "created_at: _", updated_at AS "updated_at: _"     
                    FROM p2p_nodes WHERE peer_id = ?"#,
                peer_id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List P2P nodes with filtering
    #[instrument(skip(self))]
    pub async fn list_p2p_nodes(&self, filter: &P2pNodeFilter) -> Result<Vec<P2pNode>> {
        debug!("Listing P2P nodes with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT 
                    participant_id, peer_id, node_type, multiaddr, public_key,
                    capabilities, status, last_seen, connection_quality, latency_ms,
                    metadata, created_at, updated_at
                FROM p2p_nodes"#,
        );

        let mut add_where = add_where();

        if let Some(node_type) = filter.node_type {
            add_where(&mut qb);
            qb.push("node_type = ");
            qb.push_bind(node_type as i32);
        }

        if let Some(status) = filter.status {
            add_where(&mut qb);
            qb.push("status = ");
            qb.push_bind(status as i32);
        }

        if filter.online_only.unwrap_or(false) {
            add_where(&mut qb);
            qb.push("status = 0"); // Online status
        }

        if let Some(min_quality) = filter.min_connection_quality {
            add_where(&mut qb);
            qb.push("connection_quality >= ");
            qb.push_bind(min_quality);
        }

        if let Some(max_latency) = filter.max_latency_ms {
            add_where(&mut qb);
            qb.push("latency_ms <= ");
            qb.push_bind(max_latency);
        }

        if let Some(has_public_key) = filter.has_public_key {
            if has_public_key {
                add_where(&mut qb);
                qb.push("public_key IS NOT NULL");
            } else {
                add_where(&mut qb);
                qb.push("public_key IS NULL");
            }
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            qb.push("(peer_id LIKE ");
            qb.push_bind(format!("%{search_term}%"));
            qb.push(" OR multiaddr LIKE ");
            qb.push_bind(format!("%{search_term}%"));
            qb.push(")");
        }

        if let Some(last_seen_after) = &filter.last_seen_after {
            add_where(&mut qb);
            qb.push("last_seen >= ");
            qb.push_bind(last_seen_after);
        }

        if let Some(last_seen_before) = &filter.last_seen_before {
            add_where(&mut qb);
            qb.push("last_seen <= ");
            qb.push_bind(last_seen_before);
        }

        qb.push(" ORDER BY last_seen DESC");

        if let Some(limit) = filter.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        Ok(qb
            .build_query_as::<'_, P2pNode>()
            .fetch_all(&self.pool)
            .await?)
    }

    /// Update P2P node
    #[instrument(skip(self))]
    pub async fn update_p2p_node(&self, p2p_node: &P2pNode) -> Result<()> {
        debug!(
            "Updating P2P node with peer_id: {} and participant_id: {}",
            p2p_node.peer_id, p2p_node.participant_id
        );

        let affected = sqlx::query!(
            "UPDATE p2p_nodes SET
                node_type = ?, multiaddr = ?, public_key = ?,
                capabilities = ?, status = ?, last_seen = ?, connection_quality = ?,
                latency_ms = ?, metadata = ?, updated_at = ?
             WHERE participant_id = ? AND peer_id = ?",
            p2p_node.node_type,
            p2p_node.multiaddr,
            p2p_node.public_key,
            p2p_node.capabilities,
            p2p_node.status,
            p2p_node.last_seen,
            p2p_node.connection_quality,
            p2p_node.latency_ms,
            p2p_node.metadata,
            p2p_node.updated_at,
            p2p_node.participant_id,
            p2p_node.peer_id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P node with ID {} not found",
                p2p_node.peer_id
            )));
        }

        Ok(())
    }

    /// Delete P2P node
    #[instrument(skip(self))]
    pub async fn delete_p2p_node(&self, participant_id: &Uuid, peer_id: &Uuid) -> Result<()> {
        debug!(
            "Deleting P2P node with peer_id: {} and participant_id: {}",
            peer_id, participant_id
        );

        let affected = sqlx::query!(
            "DELETE FROM p2p_nodes WHERE participant_id= ? AND peer_id = ?",
            participant_id,
            peer_id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P node with peer_id: {} and participant_id: {} not found!",
                peer_id, participant_id
            )));
        }

        Ok(())
    }

    /// Get online nodes
    #[instrument(skip(self))]
    pub async fn get_online_nodes(&self) -> Result<Vec<P2pNode>> {
        let filter = P2pNodeFilter {
            node_type: None,
            status: Some(P2pNodeStatus::Online),
            online_only: Some(true),
            min_connection_quality: None,
            max_latency_ms: None,
            has_public_key: None,
            search_term: None,
            last_seen_after: None,
            last_seen_before: None,
            limit: None,
            offset: None,
        };

        self.list_p2p_nodes(&filter).await
    }

    /// Get nodes by type
    #[instrument(skip(self))]
    pub async fn get_p2p_nodes_by_type(&self, node_type: P2pNodeType) -> Result<Vec<P2pNode>> {
        let filter = P2pNodeFilter {
            node_type: Some(node_type),
            status: None,
            online_only: None,
            min_connection_quality: None,
            max_latency_ms: None,
            has_public_key: None,
            search_term: None,
            last_seen_after: None,
            last_seen_before: None,
            limit: None,
            offset: None,
        };

        self.list_p2p_nodes(&filter).await
    }

    /// Get high-quality nodes
    #[instrument(skip(self))]
    pub async fn get_high_quality_nodes(&self, min_quality: f64) -> Result<Vec<P2pNode>> {
        let filter = P2pNodeFilter {
            node_type: None,
            status: None,
            online_only: Some(true),
            min_connection_quality: Some(min_quality),
            max_latency_ms: None,
            has_public_key: None,
            search_term: None,
            last_seen_after: None,
            last_seen_before: None,
            limit: None,
            offset: None,
        };

        self.list_p2p_nodes(&filter).await
    }

    /// Update node status
    #[instrument(skip(self))]
    pub async fn update_p2p_node_status(
        &self,
        participant_id: &Uuid,
        peer_id: &Uuid,
        status: P2pNodeStatus,
    ) -> Result<()> {
        let affected = sqlx::query!(
            "UPDATE p2p_nodes SET status = ? WHERE peer_id = ? AND participant_id = ?",
            status,
            peer_id,
            participant_id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P node with peer_id {peer_id} not found participant_id {participant_id}"
            )));
        }

        Ok(())
    }

    /// Update connection quality
    #[instrument(skip(self))]
    pub async fn update_p2p_node_connection_quality(
        &self,
        peer_id: &Uuid,
        participant_id: &Uuid,
        quality: f64,
        latency_ms: Option<i64>,
    ) -> Result<()> {
        debug!(
            "Updating connection quality for P2P node with peer_id: {peer_id} and participant_id: {participant_id}",
        );

        let affected = sqlx::query!(
            "UPDATE p2p_nodes SET connection_quality = ?, latency_ms = ? WHERE peer_id = ? AND participant_id = ?",
            quality,
            latency_ms,
            peer_id,
            participant_id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P node with peer_id {peer_id} and participant_id {participant_id} not found"
            )));
        }

        Ok(())
    }

    /// Update last seen timestamp
    #[instrument(skip(self))]
    pub async fn update_last_seen(&self, peer_id: &Uuid, participant_id: &Uuid) -> Result<()> {
        let now = Utc::now();

        let affected = sqlx::query!(
            "UPDATE p2p_nodes SET last_seen = ? WHERE peer_id = ? AND participant_id = ?",
            now,
            peer_id,
            participant_id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P node with peer_id {peer_id} not found"
            )));
        }
        Ok(())
    }

    /// Mark node as online
    #[instrument(skip(self))]
    pub async fn mark_online(&self, peer_id: &str) -> Result<()> {
        let now = Utc::now();

        let affected = sqlx::query!(
            "UPDATE p2p_nodes SET status = 0, last_seen = ?, updated_at = ? WHERE peer_id = ?",
            now,
            now,
            peer_id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P node with peer_id {peer_id} not found"
            )));
        }

        Ok(())
    }

    /// Mark node as offline
    #[instrument(skip(self))]
    pub async fn mark_offline(&self, peer_id: &str) -> Result<()> {
        let now = Utc::now();

        let affected = sqlx::query!(
            "UPDATE p2p_nodes SET status = 1, updated_at = ? WHERE peer_id = ?",
            now,
            peer_id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P node with peer_id {peer_id} not found"
            )));
        }

        Ok(())
    }

    /// Mark stale nodes as offline
    #[instrument(skip(self))]
    pub async fn mark_stale_as_offline(&self, cutoff_time: &DateTime<Utc>) -> Result<u64> {
        let now = Utc::now();

        let affected = sqlx::query!(
            "UPDATE p2p_nodes SET status = 1, updated_at = ? WHERE last_seen < ? AND status = 0",
            now,
            cutoff_time
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(affected)
    }

    /// Count nodes by status
    #[instrument(skip(self))]
    pub async fn count_by_status(&self, status: P2pNodeStatus) -> Result<i64> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as count FROM p2p_nodes WHERE status = ?",
            status
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count)
    }

    /// Count nodes by type
    #[instrument(skip(self))]
    pub async fn count_by_type(&self, node_type: P2pNodeType) -> Result<i64> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as count FROM p2p_nodes WHERE node_type = ?",
            node_type
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count)
    }

    /// Get average connection quality
    #[instrument(skip(self))]
    pub async fn get_average_connection_quality(&self) -> Result<f64> {
        let row = sqlx::query!(
            "SELECT AVG(connection_quality) as avg_quality FROM p2p_nodes WHERE status = 0"
        )
        .fetch_one(&self.pool)
        .await?;
        debug!("Average connection quality: {:?}", row.avg_quality);
        Ok(row.avg_quality.unwrap_or(0.0))
    }

    /// Get network statistics
    #[instrument(skip(self))]
    pub async fn get_network_stats(&self) -> Result<P2pNetworkStats> {
        let stats_row = sqlx::query!(
            "SELECT    
                COUNT(CASE WHEN status = 0 THEN 1 END) as online_count, 
                COUNT(CASE WHEN status = 1 THEN 1 END) as offline_count,
                COUNT(CASE WHEN status = 2 THEN 1 END) as unreachable_count,
                COUNT(CASE WHEN node_type = 0 THEN 1 END) as agent_nodes,
                COUNT(CASE WHEN node_type = 1 THEN 1 END) as gateway_nodes,
                COUNT(CASE WHEN node_type = 2 THEN 1 END) as relay_nodes,
                AVG(CASE WHEN status = 0 THEN connection_quality END) as avg_quality,
                AVG(CASE WHEN status = 0 THEN latency_ms END) as avg_latency
             FROM p2p_nodes",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(P2pNetworkStats {
            online_nodes: stats_row.online_count,
            offline_nodes: stats_row.offline_count,
            unreachable_nodes: stats_row.unreachable_count,
            agent_nodes: stats_row.agent_nodes,
            gateway_nodes: stats_row.gateway_nodes,
            relay_nodes: stats_row.relay_nodes,
            average_connection_quality: stats_row.avg_quality.unwrap_or(0.0),
            average_latency_ms: Some(stats_row.avg_latency.unwrap_or(0)),
        })
    }

    /// Delete nodes by workspace
    #[instrument(skip(self))]
    pub async fn delete_by_workspace(&self, workspace_id: &Uuid) -> Result<u64> {
        debug!("Deleting P2P nodes by workspace ID: {}", workspace_id);
        // Note: workspace_id column doesn't exist in p2p_nodes table
        // This method is kept for API compatibility but doesn't actually filter by workspace
        let affected = sqlx::query!("DELETE FROM p2p_nodes")
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected)
    }

    /// Upsert node (insert or update if peer_id exists)
    #[instrument(skip(self))]
    pub async fn upsert(&self, p2p_node: &P2pNode) -> Result<()> {
        debug!(
            "Upserting P2P node with peer_id: {} and participant_id: {}",
            p2p_node.peer_id, p2p_node.participant_id
        );

        sqlx::query!(
            "INSERT INTO p2p_nodes (
                participant_id, peer_id, node_type, multiaddr, public_key,
                capabilities, status, last_seen, connection_quality, latency_ms,
                metadata, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(participant_id, peer_id) DO UPDATE SET
                node_type = excluded.node_type,
                multiaddr = excluded.multiaddr, 
                public_key = excluded.public_key,
                capabilities = excluded.capabilities,
                status = excluded.status,
                last_seen = excluded.last_seen,
                connection_quality = excluded.connection_quality,
                latency_ms = excluded.latency_ms,
                metadata = excluded.metadata,
                updated_at = excluded.updated_at",
            p2p_node.participant_id,
            p2p_node.peer_id,
            p2p_node.node_type,
            p2p_node.multiaddr,
            p2p_node.public_key,
            p2p_node.capabilities,
            p2p_node.status,
            p2p_node.last_seen,
            p2p_node.connection_quality,
            p2p_node.latency_ms,
            p2p_node.metadata,
            p2p_node.created_at,
            p2p_node.updated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pNetworkStats {
    pub online_nodes: i64,
    pub offline_nodes: i64,
    pub unreachable_nodes: i64,
    pub agent_nodes: i64,
    pub gateway_nodes: i64,
    pub relay_nodes: i64,
    pub average_connection_quality: f64,
    pub average_latency_ms: Option<i64>,
}
