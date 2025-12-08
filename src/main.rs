use std::collections::HashMap;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufStream},
    net::{TcpListener, TcpStream},
};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Method {
    Get,
}

impl TryFrom<&str> for Method {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "GET" => Ok(Method::Get),
            m => Err(anyhow::anyhow!("unsupported method: {m}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    method: Method,
    path: String,
    headers: HashMap<String, String>,
}

impl Request {
    pub fn new(method: Method, path: String) -> Self {
        Self {
            method,
            path,
            headers: HashMap::new(),
        }
    }

    pub fn set_header(&mut self, name: String, value: String) -> Option<String> {
        self.headers.insert(name, value)
    }
}

async fn parse_request(stream: &mut BufStream<TcpStream>) -> anyhow::Result<Request> {
    let mut line_buffer = String::new();
    stream.read_line(&mut line_buffer).await?;

    let mut parts = line_buffer.split_whitespace();

    let method: Method = parts
        .next()
        .ok_or(anyhow::anyhow!("missing method"))
        .and_then(TryInto::try_into)?;

    let path: String = parts
        .next()
        .ok_or(anyhow::anyhow!("missing path"))
        .map(Into::into)?;

    let mut request = Request::new(method, path);

    loop {
        line_buffer.clear();
        stream.read_line(&mut line_buffer).await?;

        if line_buffer.is_empty() || line_buffer == "\n" || line_buffer == "\r\n" {
            break;
        }

        let mut comps = line_buffer.split(":");
        let key = comps.next().ok_or(anyhow::anyhow!("missing header name"))?;
        let value = comps
            .next()
            .ok_or(anyhow::anyhow!("missing header value"))?
            .trim();

        request.set_header(key.to_string(), value.to_string());
    }

    Ok(request)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, derive_more::Display)]
pub enum Status {
    #[display("200 OK")]
    Ok,

    #[display("404 Not Found")]
    NotFound,
}

#[derive(Debug, Clone)]
pub struct Response {
    status: Status,
    headers: HashMap<String, String>,
}

impl Response {
    pub fn new(status: Status) -> Self {
        Self {
            status,
            headers: HashMap::new(),
        }
    }

    pub fn set_header(&mut self, name: String, value: String) -> Option<String> {
        self.headers.insert(name, value)
    }

    pub async fn write(mut self, stream: &mut BufStream<TcpStream>) -> anyhow::Result<()> {
        stream
            .write_all(format!("HTTP/1.1 {}\r\n\r\n", self.status).as_bytes())
            .await?;

        //tokio::io::copy(&mut self.data, stream).await?;

        Ok(())
    }
}

async fn handle_request(
    _stream: &mut BufStream<TcpStream>,
    request: Request,
) -> anyhow::Result<Response> {
    info!("request: {:?}", request);

    let response = Response::new(Status::Ok);

    info!("response: {:?}", response);
    Ok(response)
}

async fn handle_connection(mut stream: BufStream<TcpStream>) -> anyhow::Result<()> {
    let request = parse_request(&mut stream).await?;
    let response = handle_request(&mut stream, request).await?;

    response.write(&mut stream).await?;

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
