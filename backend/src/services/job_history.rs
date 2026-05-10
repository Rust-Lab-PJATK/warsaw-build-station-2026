use crate::data::job_history::JobHistoryEntry;
use mongodb::{Client, Database, bson::doc};
use tracing::{debug, error};

pub struct JobHistoryService {
    db: Database,
}

impl JobHistoryService {
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

    /// Save a job history entry to MongoDB
    pub async fn save(
        &self,
        job_history: &JobHistoryEntry,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let collection = self.db.collection::<JobHistoryEntry>("job_history");

        debug!("Saving job history for job_id: {}", job_history.job_id);

        let result = collection.insert_one(job_history).await?;

        match result.inserted_id.as_str() {
            Some(id) => {
                debug!("Job history saved successfully with id: {}", id);
                Ok(id.to_string())
            }
            None => {
                error!("Failed to extract string ID from insert result");
                Err("Failed to extract string ID from insert result".into())
            }
        }
    }

    /// Get a job history entry by job_id
    pub async fn get_by_job_id(
        &self,
        job_id: i64,
    ) -> Result<Option<JobHistoryEntry>, Box<dyn std::error::Error>> {
        let collection = self.db.collection::<JobHistoryEntry>("job_history");

        debug!("Fetching job history for job_id: {}", job_id);

        let job_history = collection.find_one(doc! { "job_id": job_id }).await?;

        Ok(job_history)
    }

    /// Get all job history entries
    pub async fn get_all(&self) -> Result<Vec<JobHistoryEntry>, Box<dyn std::error::Error>> {
        let collection = self.db.collection::<JobHistoryEntry>("job_history");

        debug!("Fetching all job history");

        let mut cursor = collection.find(doc! {}).await?;

        let mut results = Vec::new();
        while cursor.advance().await? {
            results.push(cursor.deserialize_current()?);
        }

        Ok(results)
    }

    /// Update a job history entry
    pub async fn update(
        &self,
        job_history: &JobHistoryEntry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let collection = self.db.collection::<JobHistoryEntry>("job_history");

        debug!("Updating job history for job_id: {}", job_history.job_id);

        let mut updated_history = job_history.clone();
        updated_history.updated_at = Some(chrono::Utc::now().to_rfc3339());

        let result = collection
            .replace_one(doc! { "job_id": job_history.job_id }, &updated_history)
            .await?;

        if result.matched_count == 0 {
            return Err("Job history not found".into());
        }

        debug!("Job history updated successfully");
        Ok(())
    }

    /// Delete a job history entry by job_id
    pub async fn delete_by_job_id(&self, job_id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let collection = self.db.collection::<JobHistoryEntry>("job_history");

        debug!("Deleting job history for job_id: {}", job_id);

        let result = collection.delete_one(doc! { "job_id": job_id }).await?;

        if result.deleted_count == 0 {
            return Err("Job history not found".into());
        }

        debug!("Job history deleted successfully");
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
