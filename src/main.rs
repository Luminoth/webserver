use std::collections::HashMap;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufStream},
    net::{TcpListener, TcpStream},
};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum Method {
    #[default]
    Invalid,
    Get,
}

#[derive(Debug, Default, Clone)]
pub struct Request {
    method: Method,
    path: String,
    headers: HashMap<String, String>,
}

async fn handle_request(
    _stream: &mut BufStream<TcpStream>,
    request: Request,
) -> anyhow::Result<()> {
    info!("request: {:?}", request);

    Ok(())
}

async fn parse_request(stream: &mut BufStream<TcpStream>) -> anyhow::Result<Request> {
    let mut line_buffer = String::new();
    stream.read_line(&mut line_buffer).await?;

    let mut request = Request::default();

    Ok(request)
}

async fn handle_connection(mut stream: BufStream<TcpStream>) -> anyhow::Result<()> {
    let request = parse_request(&mut stream).await?;
    handle_request(&mut stream, request).await?;

    stream.write_all("OK".as_bytes()).await?;
    stream.flush().await?;

    Ok(())
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

    // TODO: configurable address / port
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    info!("listening at {}", listener.local_addr()?);

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("new connection from {addr}");

        tokio::spawn(async move {
            let stream = BufStream::new(stream);
            match handle_connection(stream).await {
                Ok(_) => {}
                Err(e) => {
                    println!("error: {e}");
                }
            }
        });
    }
}
