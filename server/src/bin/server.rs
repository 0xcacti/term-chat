use server::server::init;

#[tokio::main]
async fn main() {
    match init().await {
        Ok(_) => println!("Server started"),
        Err(e) => {
            eprintln!("Server error: {}", e);
            std::process::exit(1);
        }
    }
}
