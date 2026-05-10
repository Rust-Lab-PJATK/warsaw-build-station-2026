use backend::app::App;
use loco_rs::cli;

#[tokio::main]
#[allow(clippy::result_large_err)]
async fn main() -> loco_rs::Result<()> {
    dotenvy::dotenv().expect("Failed to load .env file");
    cli::main::<App>().await
}
