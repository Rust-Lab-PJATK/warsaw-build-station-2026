use async_trait::async_trait;
use loco_rs::app::{AppContext, Initializer};
use loco_rs::Result;
use mongodb::Client;
use tracing::info;

pub struct MongoDbInitializer;

#[async_trait]
impl Initializer for MongoDbInitializer {
    fn name(&self) -> String {
        "mongodb".to_string()
    }

    async fn before_run(&self, _ctx: &AppContext) -> Result<()> {
        info!("Initializing MongoDB connection...");

        let mongo_url = std::env::var("MONGODB_URL")
            .unwrap_or_else(|_| "mongodb://loco:loco@localhost:27017".to_string());

        let client = Client::connect(mongo_url).await?;
        let db = client.database("loco_dev");

        db.run_command(mongodb::bson::doc! { "ping": 1 }, None).await?;

        info!("MongoDB connection established successfully");

        Ok(())
    }
}
