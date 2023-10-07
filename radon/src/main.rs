use clap::{crate_version, Parser, Subcommand};
use figment::{providers::Env, Figment};
use server::server::ServerConfig;
use std::{env, process};

#[derive(Debug, Parser)]
#[command(name="term-chat-server", version=crate_version!(), about="terminal chat server", long_about = "Server to let you chat with friends in the terminal", arg_required_else_help(true))]
struct TermChatParser {
    /// The subcommand to run
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run,
}

#[tokio::main]
async fn main() {
    let mut config: ServerConfig = Figment::new()
        .merge(Env::prefixed("SERVER_"))
        .extract()
        .unwrap();

    let args = TermChatParser::parse();

    // handle commands
    match &args.command {
        Some(Commands::Run) => {
            let server_address = "127.0.0.1:8000";
            let server = server::Server::new(&server_address).await.unwrap();
            server.run().await.unwrap();
        }
        None => {
            eprintln!("No command provided");
            process::exit(1);
        }
    }
}
