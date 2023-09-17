pub mod error;

use tokio::sync::broadcast;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};

use self::error::ServerError;

pub async fn init() -> Result<(), ServerError> {
    let listener = TcpListener::bind("localhost:8080")
        .await
        .map_err(ServerError::TcpBind)?;

    let (tx, _) = broadcast::channel(10);
    loop {
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        let (mut socket, addr) = listener.accept().await.map_err(ServerError::TcpAccept)?;
        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        match result {
                            Ok(n) if n == 0 => break,
                            Ok(_) => {
                                if tx.send((line.clone(), addr)).is_err() {
                                    break;
                                }
                                line.clear()
                            }
                            Err(e) => {
                                eprintln!("failed to read from socket; err = {:?}", e);
                                break;
                            }

                        }
                    }
                    result = rx.recv() => {
                        match result {
                            Ok((msg, other_addr)) if addr != other_addr => {
                                if writer.write_all(msg.as_bytes()).await.is_err() {
                                    eprintln!("failed to write to socket");
                                    break;
                                }

                            }
                            Err(broadcast::error::RecvError::Lagged(_)) => {
                                eprintln!("lagged");
                                break;
                            }
                            Err(broadcast::error::RecvError::Closed) => {
                                eprintln!("channel closed");
                                break;
                            }

                            _ => (),

                        }
                    }
                }
            }
        });
    }
}
