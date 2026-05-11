//! JobService — Internal service for managing jobs
//!
//! This service is for backend-to-backend communication only.
//! Endpoints and external access are NOT exposed.

use crate::data::job::Job;
use mongodb::{Client, Database, bson::doc};
use tracing::{debug, error};

pub struct JobService {
    db: Database,
}

impl JobService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get database client from environment and return a service instance
    pub async fn from_env()
    -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mongo_url = std::env::var("MONGODB_URL").unwrap_or_else(|_| {
            "mongodb://loco:loco@localhost:27017/loco_dev?authSource=admin".to_string()
        });

        let options = mongodb::options::ClientOptions::parse(&mongo_url).await?;
        let client = Client::with_options(options)?;

        let db_name = std::env::var("MONGODB_DB_NAME").unwrap_or_else(|_| "loco_dev".to_string());
        let db = client.database(&db_name);

        Ok(Self::new(db))
    }

    /// Save a job to MongoDB (jobs collection)
    pub async fn save(&self, job: &Job)
    -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let collection = self.db.collection::<Job>("jobs");

        debug!("Saving job");

        let result = collection.insert_one(job).await?;

        match result.inserted_id.as_str() {
            Some(id) => {
                debug!("Job saved successfully with id: {}", id);
                Ok(id.to_string())
            }
            None => {
                error!("Failed to extract string ID from insert result");
                Err("Failed to extract string ID from insert result".into())
            }
        }
    }

    /// Get a job by ID
    pub async fn get_by_id(&self, id: &str)
    -> Result<Option<Job>, Box<dyn std::error::Error + Send + Sync>> {
        let collection = self.db.collection::<Job>("jobs");

        debug!("Fetching job by id: {}", id);

        let object_id = mongodb::bson::oid::ObjectId::parse_str(id)?;
        let job = collection.find_one(doc! { "_id": object_id }).await?;

        Ok(job)
    }

    /// Get a job by task pubkey
    pub async fn get_by_task_pubkey(
        &self,
        task_pubkey: &str,
    ) -> Result<Option<Job>, Box<dyn std::error::Error + Send + Sync>> {
        let collection = self.db.collection::<Job>("jobs");

        debug!("Fetching job by task_pubkey: {}", task_pubkey);

        let job = collection.find_one(doc! { "task_pubkey": task_pubkey }).await?;

        Ok(job)
    }

    /// Get all jobs
    pub async fn get_all(&self)
    -> Result<Vec<Job>, Box<dyn std::error::Error + Send + Sync>> {
        let collection = self.db.collection::<Job>("jobs");

        debug!("Fetching all jobs");

        let mut cursor = collection.find(doc! {}).await?;

        let mut results = Vec::new();
        while cursor.advance().await? {
            results.push(cursor.deserialize_current()?);
        }

        Ok(results)
    }

    /// Update a job by ID
    pub async fn update_by_id(
        &self,
        id: &str,
        job: &Job,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let collection = self.db.collection::<Job>("jobs");

        debug!("Updating job by id: {}", id);

        let mut updated_job = job.clone();
        updated_job.updated_at = Some(chrono::Utc::now().to_rfc3339());

        let object_id = mongodb::bson::oid::ObjectId::parse_str(id)?;
        let result = collection
            .replace_one(doc! { "_id": object_id }, &updated_job)
            .await?;

        if result.matched_count == 0 {
            return Err("Job not found".into());
        }

        debug!("Job updated successfully");
        Ok(())
    }

    /// Update a job by task pubkey
    pub async fn update_by_task_pubkey(
        &self,
        task_pubkey: &str,
        job: &Job,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let collection = self.db.collection::<Job>("jobs");

        debug!("Updating job by task_pubkey: {}", task_pubkey);

        let mut updated_job = job.clone();
        updated_job.updated_at = Some(chrono::Utc::now().to_rfc3339());

        let result = collection
            .replace_one(doc! { "task_pubkey": task_pubkey }, &updated_job)
            .await?;

        if result.matched_count == 0 {
            return Err("Job not found".into());
        }

        debug!("Job updated successfully");
        Ok(())
    }

    /// Delete a job by ID
    pub async fn delete_by_id(&self, id: &str)
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let collection = self.db.collection::<Job>("jobs");

        debug!("Deleting job by id: {}", id);

        let object_id = mongodb::bson::oid::ObjectId::parse_str(id)?;
        let result = collection.delete_one(doc! { "_id": object_id }).await?;

        if result.deleted_count == 0 {
            return Err("Job not found".into());
        }

        debug!("Job deleted successfully");
        Ok(())
    }
}

// Backward compatibility alias
pub type JobHistoryService = JobService;
