use tokio::io::{AsyncBufReadExt, BufReader};

use tokio::sync::broadcast;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    let (tx, rx) = broadcast::channel::<String>(10);
    loop {
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        let (mut socket, addr) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                    tx.send(line.clone()).unwrap();
                line.clear()

                    }
                }
                let bytes_read = reader.read_line(&mut line).await.unwrap();
                if bytes_read == 0 {
                    break;
                }
                tx.send(line.clone()).unwrap();
                let msg = rx.recv().await.unwrap();
                writer.write_all(msg.as_bytes()).await.unwrap();
            }
        });
    }
}
