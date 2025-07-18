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

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum AddressType {
    Home = 0,
    Work = 1,
    Other = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum AddressStatus {
    Active = 0,
    Archived = 1,
    Deleted = 2,
}
/// Address model
#[boilermates("CreateModel")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    #[boilermates(not_in("CreateModel"))]
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub address_type: AddressType,
    pub status: AddressStatus,
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub metadata: Option<Json<Value>>,
    #[boilermates(not_in("CreateModel"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateModel"))]
    pub updated_at: DateTime<Utc>,
}

/// Additional filtering options for address queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
pub struct AddressFilter {
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub search_term: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new address in the database
    #[instrument(skip(self))]
    pub async fn create_address(&self, address: &Address) -> Result<Address> {
        let id = Uuid::new_v4();
        debug!("Creating address with ID: {}", address.id);

        Ok(sqlx::query_as!(
            Address,
            r#"INSERT INTO addresses (
                    id, user_id, contact_id, account_id, address_type, status,
                    street, city, state, postal_code, country, country_code, latitude, longitude,
                    metadata, created_at, updated_at
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                ) RETURNING
                    id AS "id: _", user_id AS "user_id: _", contact_id AS "contact_id: _",
                    account_id AS "account_id: _",
                    address_type AS "address_type: AddressType", status as "status: AddressStatus",
                    street, city, state, postal_code, country, country_code, latitude, longitude,
                    metadata AS "metadata: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _""#,
            id,
            address.user_id,
            address.contact_id,
            address.account_id,
            address.address_type,
            address.status,
            address.street,
            address.city,
            address.state,
            address.postal_code,
            address.country,
            address.country_code,
            address.latitude,
            address.longitude,
            address.metadata,
            address.created_at,
            address.updated_at
        )
        .fetch_one(&self.pool)
        .await?)
    }

    #[instrument(skip(self))]
    pub async fn get_address_by_id(&self, id: &Uuid) -> Result<Option<Address>> {
        debug!("Getting address by ID: {}", id);
        debug!("Getting address by ID: {:?}", id);

        Ok(sqlx::query_as!(
            Address,
            r#"SELECT 
                    id AS "id: _", user_id AS "user_id: _", contact_id AS "contact_id: _",
                    account_id AS "account_id: _",
                    address_type AS "address_type: AddressType", status as "status: AddressStatus", street, city, state,
                    postal_code, country, country_code, latitude, longitude, metadata AS "metadata: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM addresses WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List and filter addresses
    #[instrument(err, skip(self, filter))]
    pub async fn list_addresses(&self, filter: &AddressFilter) -> Result<Vec<Address>> {
        debug!("Listing addresses with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT
                    id AS "id: _", user_id AS "user_id: _", contact_id AS "contact_id: _",
                    account_id AS "account_id: _",
                    address_type AS "address_type: AddressType", status as "status: AddressStatus", street, city, state,
                    postal_code, country, country_code, latitude, longitude, metadata AS "metadata: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM addresses"#,
        );

        let mut first_condition = true;
        let mut add_where = |qb: &mut QueryBuilder<Sqlite>| {
            if first_condition {
                qb.push(" WHERE ");
                first_condition = false;
            } else {
                qb.push(" AND ");
            }
        };

        if let Some(city) = &filter.city {
            add_where(&mut qb);
            qb.push_bind(format!("%{city}%"));
        }

        if let Some(state) = &filter.state {
            add_where(&mut qb);
            qb.push_bind(format!("%{state}%"));
        }

        if let Some(country) = &filter.country {
            add_where(&mut qb);
            qb.push_bind(format!("%{country}%"));
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            qb.push_bind(format!("%{search_term}%"));
        }

        qb.push(" ORDER BY updated_at DESC");

        if let Some(limit) = filter.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        let rows = qb.build_query_as::<Address>().fetch_all(&self.pool).await?;

        Ok(rows)
    }

    /// Update an address
    #[instrument(skip(self))]
    pub async fn update_address(&self, address: &Address) -> Result<Address> {
        debug!("Updating address with ID: {}", address.id);
        let metadata = address.metadata.as_deref();

        Ok(sqlx::query_as!(
            Address,
            r#"UPDATE addresses SET 
                user_id = ?, contact_id = ?, account_id = ?,
                address_type = ?, status = ?, street = ?, city = ?, state = ?, postal_code = ?,
                country = ?, country_code = ?, latitude = ?, longitude = ?,
                metadata = ?, updated_at = ?
            WHERE id = ? RETURNING
                id AS "id: _", user_id AS "user_id: _", contact_id AS "contact_id: _",
                account_id AS "account_id: _",
                address_type AS "address_type: AddressType", status as "status: AddressStatus", street, city, state,
                postal_code, country, country_code, latitude, longitude, metadata AS "metadata: _",
                created_at AS "created_at: _", updated_at AS "updated_at: _""#,
            address.user_id,
            address.contact_id,
            address.account_id,
            address.address_type,
            address.status,
            address.street,
            address.city,
            address.state,
            address.postal_code,
            address.country,
            address.country_code,
            address.latitude,
            address.longitude,
            metadata,
            address.updated_at,
            address.id
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Delete an address by ID
    #[instrument(err, skip(self))]
    pub async fn delete_address(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting address with ID: {}", id);

        let affected = sqlx::query("DELETE FROM addresses WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Address with ID {id} not found for delete"
            )));
        }

        Ok(())
    }
}
