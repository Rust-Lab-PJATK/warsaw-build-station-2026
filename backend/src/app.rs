use async_trait::async_trait;
use loco_rs::{
    Result,
    app::{AppContext, Hooks, Initializer},
    bgworker::{BackgroundWorker, Queue},
    boot::{BootResult, StartMode, create_app},
    config::Config,
    controller::AppRoutes,
    environment::Environment,
    task::Tasks,
};

#[allow(unused_imports)]
use crate::{
    controllers, initializers::MongoDbInitializer, tasks, workers::downloader::DownloadWorker,
};

pub struct App;
#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        // Early validation of estimate provider configuration to fail fast with clear message.
        // This prevents runtime 503s caused by missing provider envs and helps operators fix config quickly.
        // if let Err(err) = validate_provider_env() {
        //     eprintln!("Estimate provider configuration error: {}", err);
        //     // Exit process with non-zero code so orchestrators notice startup failure.
        //     std::process::exit(1);
        // }

        create_app::<Self>(mode, environment, config).await
    }

    // fn validate_provider_env() -> Result<(), String> {
    //     // Default provider is elevenlabs when unset or empty.
    //     let provider = env::var("ESTIMATE_PROVIDER").unwrap_or_else(|_| "elevenlabs".to_string());
    //     let normalized = provider.trim().to_ascii_lowercase();
    //
    //     match normalized.as_str() {
    //         "elevenlabs" => {
    //             let key = env::var("ELEVENLABS_API_KEY").unwrap_or_default();
    //             if key.trim().is_empty() {
    //                 return Err("missing ELEVENLABS_API_KEY environment variable".to_string());
    //             }
    //             let agent = env::var("ELEVENLABS_AGENT_ID").unwrap_or_default();
    //             if agent.trim().is_empty() {
    //                 return Err("missing ELEVENLABS_AGENT_ID environment variable".to_string());
    //             }
    //             Ok(())
    //         }
    //         "openai" => {
    //             let key = env::var("OPENAI_API_KEY").unwrap_or_default();
    //             if key.trim().is_empty() {
    //                 return Err("missing OPENAI_API_KEY environment variable".to_string());
    //             }
    //             Ok(())
    //         }
    //         other => Err(format!(
    //             "invalid ESTIMATE_PROVIDER value: '{}' (supported: 'elevenlabs' or 'openai')",
    //             other
    //         )),
    //     }
    // }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![Box::new(MongoDbInitializer)])
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes() // controller routes below
            .add_route(controllers::home::routes())
            .add_route(controllers::estimate::routes())
            .add_route(controllers::job_history::routes())
    }
    async fn connect_workers(ctx: &AppContext, queue: &Queue) -> Result<()> {
        queue.register(DownloadWorker::build(ctx)).await?;
        Ok(())
    }

    #[allow(unused_variables)]
    fn register_tasks(tasks: &mut Tasks) {
        // tasks-inject (do not remove)
    }
}
