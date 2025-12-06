use std::collections::HashMap;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Method {
    Get,
}

#[derive(Debug, Clone)]
pub struct Requeet {
    method: Method,
    path: String,
    headers: HashMap<String, String>,
}

async fn handle_request(mut socket: TcpStream) {
    info!("handling request");
    let mut buf = vec![0; 1024];

    loop {
        match socket.read(&mut buf).await {
            Ok(0) => {
                info!("remote connection closed");
                return;
            }
            Ok(n) => {
                // TODO: replace with handling the data
                if socket.write_all(&buf[..n]).await.is_err() {
                    info!("unexpected socket error while writing");
                    return;
                }
            }
            Err(_) => {
                info!("unexpected socket error while reading");
                return;
            }
        }
    }
}

fn init_logging() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging()?;

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    info!("listening at {}", listener.local_addr().unwrap());

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            handle_request(socket).await;
        });
    }
}
