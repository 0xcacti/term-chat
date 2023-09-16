pub mod error;

use tokio::sync::broadcast;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};

#[tokio::main]
async fn init() -> Result<(), ServerError> {
    let listener = TcpListener::bind("localhost:8080").await?;
    let (tx, rx) = broadcast::channel(10);
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
                        if result.unwrap() == 0 {
                            break;
                        }

                        tx.send((line.clone(), addr)).unwrap();
                        line.clear()
                    }
                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();
                        if addr != other_addr {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
