//! ChatHistoryService — Internal service for recording chat sessions
//!
//! This service is for backend-to-backend communication only.
//! Endpoints and external access are NOT exposed.

use crate::data::chat_history::ChatHistoryEntry;
use mongodb::{Client, Database, bson::doc};
use tracing::{debug, error};

pub struct ChatHistoryService {
    db: Database,
}

impl ChatHistoryService {
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

    /// Save a chat history entry to MongoDB (chat_history collection)
    pub async fn save(
        &self,
        entry: &ChatHistoryEntry,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let collection = self.db.collection::<ChatHistoryEntry>("chat_history");

        debug!(
            "Saving chat history for wallet: {}, job_id: {}",
            entry.wallet_address, entry.job_id
        );

        let result = collection.insert_one(entry).await?;

        match result.inserted_id.as_str() {
            Some(id) => {
                debug!("Chat history saved successfully with id: {}", id);
                Ok(id.to_string())
            }
            None => {
                error!("Failed to extract string ID from insert result");
                Err("Failed to extract string ID from insert result".into())
            }
        }
    }

    /// Get chat history entry by ID
    pub async fn get_by_id(
        &self,
        id: &str,
    ) -> Result<Option<ChatHistoryEntry>, Box<dyn std::error::Error>> {
        let collection = self.db.collection::<ChatHistoryEntry>("chat_history");

        debug!("Fetching chat history by id: {}", id);

        let object_id = mongodb::bson::oid::ObjectId::parse_str(id)?;
        let entry = collection.find_one(doc! { "_id": object_id }).await?;

        Ok(entry)
    }

    /// Get all chat history entries for a wallet
    pub async fn get_by_wallet(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<ChatHistoryEntry>, Box<dyn std::error::Error>> {
        let collection = self.db.collection::<ChatHistoryEntry>("chat_history");

        debug!("Fetching chat history for wallet: {}", wallet_address);

        let mut cursor = collection
            .find(doc! { "wallet_address": wallet_address })
            .await?;

        let mut results = Vec::new();
        while cursor.advance().await? {
            results.push(cursor.deserialize_current()?);
        }

        Ok(results)
    }

    /// Get all chat history entries for a specific job
    pub async fn get_by_job_id(
        &self,
        job_id: i64,
    ) -> Result<Vec<ChatHistoryEntry>, Box<dyn std::error::Error>> {
        let collection = self.db.collection::<ChatHistoryEntry>("chat_history");

        debug!("Fetching chat history for job_id: {}", job_id);

        let mut cursor = collection.find(doc! { "job_id": job_id }).await?;

        let mut results = Vec::new();
        while cursor.advance().await? {
            results.push(cursor.deserialize_current()?);
        }

        Ok(results)
    }

    /// Get all chat history entries
    pub async fn get_all(&self) -> Result<Vec<ChatHistoryEntry>, Box<dyn std::error::Error>> {
        let collection = self.db.collection::<ChatHistoryEntry>("chat_history");

        debug!("Fetching all chat history");

        let mut cursor = collection.find(doc! {}).await?;

        let mut results = Vec::new();
        while cursor.advance().await? {
            results.push(cursor.deserialize_current()?);
        }

        Ok(results)
    }

    /// Update a chat history entry
    pub async fn update(&self, entry: &ChatHistoryEntry) -> Result<(), Box<dyn std::error::Error>> {
        let collection = self.db.collection::<ChatHistoryEntry>("chat_history");

        debug!("Updating chat history for job_id: {}", entry.job_id);

        let mut updated_entry = entry.clone();
        updated_entry.updated_at = Some(chrono::Utc::now().to_rfc3339());

        let object_id =
            mongodb::bson::oid::ObjectId::parse_str(entry.id.as_ref().ok_or("Missing entry ID")?)?;

        let result = collection
            .replace_one(doc! { "_id": object_id }, &updated_entry)
            .await?;

        if result.matched_count == 0 {
            return Err("Chat history entry not found".into());
        }

        debug!("Chat history updated successfully");
        Ok(())
    }

    /// Delete a chat history entry by ID
    pub async fn delete_by_id(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let collection = self.db.collection::<ChatHistoryEntry>("chat_history");

        debug!("Deleting chat history by id: {}", id);

        let object_id = mongodb::bson::oid::ObjectId::parse_str(id)?;
        let result = collection.delete_one(doc! { "_id": object_id }).await?;

        if result.deleted_count == 0 {
            return Err("Chat history entry not found".into());
        }

        debug!("Chat history deleted successfully");
        Ok(())
    }
}
