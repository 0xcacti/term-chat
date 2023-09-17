use server::server::Server;

#[tokio::main]
async fn main() {
    let server = Server::new("localhost:8080").await;
    match server {
        Ok(mut server) => {
            println!("server created, starting");
            match server.run().await {
                Ok(_) => {
                    println!("server finished");
                }
                Err(e) => {
                    eprintln!("server failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("failed to create server: {}", e);
            std::process::exit(1);
        }
    }
}
