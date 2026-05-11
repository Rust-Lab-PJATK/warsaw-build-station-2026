use backend::app::App;
use loco_rs::cli;

#[tokio::main]
#[allow(clippy::result_large_err)]
async fn main() -> loco_rs::Result<()> {
    dotenvy::dotenv().expect("Failed to load .env file");

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install default rustls crypto provider");

    cli::main::<App>().await
}
