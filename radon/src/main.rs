use clap::{crate_version, Parser, Subcommand};
use radon::{
    api::{self, AppState},
    config::{RunArgs, ServerConfig},
};
use sqlx::postgres::PgPoolOptions;
use std::{env, process};

#[derive(Debug, Parser)]
#[command(name="radon", version=crate_version!(), about="terminal chat server", long_about = "Server to let you chat with friends in the terminal", arg_required_else_help(true))]
struct App {
    /// The subcommand to run
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run(RunArgs),
}

#[tokio::main]
async fn main() {
    let mut config = ServerConfig::from(ServerConfig::figment()).unwrap();
    // let database_url = dotenvy::var("DATABASE_URL").unwrap();
    // The error from `var()` doesn't mention the environment variable.

    let args = App::parse();

    // handle commands
    match &args.command {
        Some(Commands::Run(arguments)) => {
            config.merge_with_args(arguments);

            let database_url = config.db_connection_string;
            let db = PgPoolOptions::new()
                .max_connections(20)
                .connect(&database_url)
                .await
                .unwrap();

            let app_state = AppState {
                config,
                db: db.clone(),
            };

            sqlx::migrate!().run(&db).await.unwrap();
            api::run(Arc::new(app_state)).await;
        }
        None => {
            eprintln!("No command provided");
            process::exit(1);
        }
    }
}
