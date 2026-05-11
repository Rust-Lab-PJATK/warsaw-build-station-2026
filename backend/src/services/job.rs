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
    pub async fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
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
    pub async fn save(&self, job: &Job) -> Result<String, Box<dyn std::error::Error>> {
        let collection = self.db.collection::<Job>("jobs");

        debug!("Saving job for job_id: {}", job.job_id);

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

    /// Get a job by job_id
    pub async fn get_by_job_id(
        &self,
        job_id: i64,
    ) -> Result<Option<Job>, Box<dyn std::error::Error>> {
        let collection = self.db.collection::<Job>("jobs");

        debug!("Fetching job for job_id: {}", job_id);

        let job = collection.find_one(doc! { "job_id": job_id }).await?;

        Ok(job)
    }

    /// Get all jobs
    pub async fn get_all(&self) -> Result<Vec<Job>, Box<dyn std::error::Error>> {
        let collection = self.db.collection::<Job>("jobs");

        debug!("Fetching all jobs");

        let mut cursor = collection.find(doc! {}).await?;

        let mut results = Vec::new();
        while cursor.advance().await? {
            results.push(cursor.deserialize_current()?);
        }

        Ok(results)
    }

    /// Update a job
    pub async fn update(&self, job: &Job) -> Result<(), Box<dyn std::error::Error>> {
        let collection = self.db.collection::<Job>("jobs");

        debug!("Updating job for job_id: {}", job.job_id);

        let mut updated_job = job.clone();
        updated_job.updated_at = Some(chrono::Utc::now().to_rfc3339());

        let result = collection
            .replace_one(doc! { "job_id": job.job_id }, &updated_job)
            .await?;

        if result.matched_count == 0 {
            return Err("Job not found".into());
        }

        debug!("Job updated successfully");
        Ok(())
    }

    /// Delete a job by job_id
    pub async fn delete_by_job_id(&self, job_id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let collection = self.db.collection::<Job>("jobs");

        debug!("Deleting job for job_id: {}", job_id);

        let result = collection.delete_one(doc! { "job_id": job_id }).await?;

        if result.deleted_count == 0 {
            return Err("Job not found".into());
        }

        debug!("Job deleted successfully");
        Ok(())
    }

    /// Get next sequential job_id (counter-based approach)
    pub async fn get_next_job_id(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let collection = self.db.collection::<mongodb::bson::Document>("counters");

        debug!("Getting next job_id counter");

        let result = collection
            .find_one_and_update(doc! { "_id": "job_id" }, doc! { "$inc": { "seq": 1i64 } })
            .await?;

        let job_id = match result {
            Some(doc) => doc.get_i64("seq").unwrap_or(1),
            None => 1,
        };

        debug!("Next job_id is: {}", job_id);
        Ok(job_id)
    }
}

// Backward compatibility alias
pub type JobHistoryService = JobService;
