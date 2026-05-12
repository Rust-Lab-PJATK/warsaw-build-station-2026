use async_trait::async_trait;
use loco_rs::app::{AppContext, Initializer};
use loco_rs::Result;
use mongodb::{Client, options::ClientOptions};
use tracing::{info, warn};

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

        let options = match ClientOptions::parse(&mongo_url).await {
            Ok(opts) => opts,
            Err(err) => {
                warn!("MongoDB connection skipped: failed to parse URL: {err}");
                return Ok(());
            }
        };

        let client = match Client::with_options(options) {
            Ok(c) => c,
            Err(err) => {
                warn!("MongoDB connection skipped: failed to create client: {err}");
                return Ok(());
            }
        };

        let db_name = std::env::var("MONGODB_DB_NAME").unwrap_or_else(|_| "loco_dev".to_string());
        let db = client.database(&db_name);

        match db.run_command(mongodb::bson::doc! { "ping": 1 }).await {
            Ok(_) => info!("MongoDB connection initialized successfully"),
            Err(err) => warn!("MongoDB unavailable, continuing without it: {err}"),
        }

        Ok(())
    }
}
