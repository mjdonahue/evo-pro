use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use serde_json::Value;
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::{QueryBuilder, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;
/// Contact model matching the SQLite schema
#[boilermates("CreateContact")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    #[boilermates(not_in("CreateContact"))]
    pub id: Uuid,
    pub name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub mobile_phone: Option<String>,
    pub home_phone: Option<String>,
    pub work_phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub job_title: Option<String>,
    pub company: Option<String>,
    pub department: Option<String>,
    pub primary_address_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub metadata: Option<Json<Value>>,   
    #[boilermates(not_in("CreateContact"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateContact"))]
    pub updated_at: DateTime<Utc>,
}

/// Additional filtering options for contact queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
pub struct ContactFilter {
    pub workspace_id: Option<Uuid>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub search_term: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new contact in the database
    #[instrument(skip(self))]
    pub async fn create_contact(&self, contact: &CreateContact) -> Result<Contact> {
        let id = Uuid::new_v4();
        debug!("Creating contact with ID: {}", id);

        Ok(sqlx::query_as!(
            Contact,
            r#"INSERT INTO contacts ( 
                    id, name, first_name, last_name, mobile_phone, home_phone, 
                    work_phone, email, website, job_title, company, department,
                    primary_address_id, workspace_id, metadata
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                ) RETURNING 
                 id AS "id: _", 
                 name, 
                 first_name, 
                 last_name, 
                 mobile_phone,
                home_phone, 
                work_phone, 
                email, 
                website, 
                job_title, 
                company, 
                department,
                primary_address_id AS "primary_address_id: _",
                workspace_id AS "workspace_id: _", 
                metadata AS "metadata: _", 
                created_at AS "created_at: _",
                updated_at AS "updated_at: _"
            "#,
            id,
            contact.name,
            contact.first_name,
            contact.last_name,
            contact.mobile_phone,
            contact.home_phone,
            contact.work_phone,
            contact.email,
            contact.website,
            contact.job_title,
            contact.company,
            contact.department,
            contact.primary_address_id,
            contact.workspace_id,
            contact.metadata,
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Get a contact by ID
    #[instrument(skip(self))]
    pub async fn get_contact_by_id(&self, id: &Uuid) -> Result<Option<Contact>> {
        debug!("Getting contact by ID: {}", id);

        Ok(sqlx::query_as!( 
            Contact,
            r#"SELECT 
                    id AS "id: _", name, first_name, last_name, mobile_phone, home_phone,
                    work_phone, email, website, job_title, company, department,
                    primary_address_id AS "primary_address_id: _", workspace_id As "workspace_id: _",
                    metadata AS "metadata: _", created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM contacts WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List and filter contacts
    #[instrument(err, skip(self, filter))]
    pub async fn list_contacts(&self, filter: &ContactFilter) -> Result<Vec<Contact>> {
        debug!("Listing contacts with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT id, name, first_name, last_name, mobile_phone, home_phone,
               work_phone, email, website, job_title, company, department,
               primary_address_id, workspace_id,
               metadata, created_at, updated_at FROM contacts"#,
        );

        let mut add_where = add_where();

        if let Some(workspace_id) = &filter.workspace_id {
            add_where(&mut qb);
            qb.push("workspace_id = ");
            qb.push_bind(workspace_id);
        }

        if let Some(name) = &filter.name {
            add_where(&mut qb);
            let pattern = format!("%{name}%");
            qb.push("(name LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR first_name LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR last_name LIKE ");
            qb.push_bind(pattern);
            qb.push(")");
        }

        if let Some(email) = &filter.email {
            add_where(&mut qb);
            qb.push("email LIKE ");
            qb.push_bind(format!("%{email}%"));
        }

        if let Some(phone) = &filter.phone {
            add_where(&mut qb);
            let pattern = format!("%{phone}%");
            qb.push("(mobile_phone LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR home_phone LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR work_phone LIKE ");
            qb.push_bind(pattern);
            qb.push(")");
        }

        if let Some(company) = &filter.company {
            add_where(&mut qb);
            qb.push("company LIKE ");
            qb.push_bind(format!("%{company}%"));
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            let pattern = format!("%{search_term}%");
            qb.push("(name LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR first_name LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR last_name LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR email LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR company LIKE ");
            qb.push_bind(pattern);
            qb.push(")");
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

        Ok(qb
            .build_query_as::<'_, Contact>()
            .fetch_all(&self.pool)
            .await?)
    }

    /// Update a contact
    #[instrument(skip(self))]
    pub async fn update_contact(&self, contact: &Contact) -> Result<()> {
        debug!("Updating contact with ID: {}", contact.id);

        let affected = sqlx::query!(
            "UPDATE contacts SET 
                name = ?, first_name = ?, last_name = ?, mobile_phone = ?, home_phone = ?,
                work_phone = ?, email = ?, website = ?, job_title = ?, company = ?, department = ?,
                primary_address_id = ?, workspace_id = ?, metadata = ?
            WHERE id = ?",
            contact.name,
            contact.first_name,
            contact.last_name,
            contact.mobile_phone,
            contact.home_phone,
            contact.work_phone,
            contact.email,
            contact.website,
            contact.job_title,
            contact.company,
            contact.department,
            contact.primary_address_id,
            contact.workspace_id,
            contact.metadata,
            contact.id,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Contact with ID {} not found for update",
                contact.id
            )));
        }

        Ok(())
    }

    /// Delete a contact by ID
    #[instrument(err, skip(self))]  
    pub async fn delete_contact(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting contact with ID: {}", id);

        let affected = sqlx::query!("DELETE FROM contacts WHERE id = ?", id)
        
            .execute(&self.pool)
            .await?
            .rows_affected();   

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
             "Contact with ID {id} not found for delete" 
            )));    
        }

        Ok(())
    }
}
