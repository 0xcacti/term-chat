use server::server;
#[tokio::main]
async fn main() {
    server::new("127.0.0.1:8080");
}
