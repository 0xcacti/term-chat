use clap::{crate_version, Parser, Subcommand};
use client::{connect, Config};
use figment::{providers::Env, Figment};
use std::{env, process};

/// term-chat client ui
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
    let mut config: Config = Figment::new()
        .merge(Env::prefixed("BIBLE_RS_"))
        .extract()
        .unwrap();

    let args = TermChatParser::parse();

    // handle commands
    match &args.command {
        Some(Commands::Auth { username, password }) => {
            config.username = Some(username.to_string());
            config.password = Some(password.to_string());
        }
        Some(Commands::Send { message }) => {
            connect().await.unwrap();
        }
        None => {
            eprintln!("No command provided");
            process::exit(1);
        }
    }
}
