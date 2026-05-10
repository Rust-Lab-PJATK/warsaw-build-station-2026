use async_trait::async_trait;
use loco_rs::app::{AppContext, Initializer};
use loco_rs::{Error, Result};
use mongodb::{Client, options::ClientOptions};
use tracing::info;

pub struct MongoDbInitializer;

#[async_trait]
impl Initializer for MongoDbInitializer {
    fn name(&self) -> String {
        "mongodb".to_string()
    }

    async fn before_run(&self, _ctx: &AppContext) -> Result<()> {
        info!("Initializing MongoDB connection...");

        let mongo_url = std::env::var("MONGODB_URL").unwrap_or_else(|_| {
            "mongodb://loco:loco@localhost:27017/loco_dev?authSource=admin".to_string()
        });

        let options = ClientOptions::parse(&mongo_url).await.map_err(Error::msg)?;

        let client = Client::with_options(options).map_err(Error::msg)?;

        let db_name = std::env::var("MONGODB_DB_NAME").unwrap_or_else(|_| "loco_dev".to_string());
        let db = client.database(&db_name);

        db.run_command(mongodb::bson::doc! { "ping": 1 })
            .await
            .map_err(Error::msg)?;

        info!("MongoDB connection initialized successfully");

        Ok(())
    }
}
