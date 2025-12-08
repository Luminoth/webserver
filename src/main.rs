use std::collections::HashMap;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::timeout,
};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

const READ_TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_millis(500);
const WRITE_TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_millis(500);
const MAX_HEADER_SIZE: usize = 1024 * 8;

#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::Display)]
pub enum Method {
    Get,
}

impl TryFrom<&str> for Method {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "GET" => Ok(Method::Get),
            m => anyhow::bail!("unsupported method: {m}"),
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

fn next_line_break(buf: &[u8]) -> Option<usize> {
    if buf.len() < 2 {
        return None;
    }

    let mut idx = 0;
    loop {
        if idx >= buf.len() - 1 {
            return None;
        }

        if buf[idx] == '\r' as u8 && buf[idx + 1] == '\n' as u8 {
            return Some(idx);
        }

        idx += 1;
    }
}

async fn read_request(stream: &mut TcpStream) -> anyhow::Result<Request> {
    let mut buf = [0; MAX_HEADER_SIZE];
    let n = match timeout(READ_TIMEOUT, stream.read(&mut buf)).await {
        Ok(Ok(n)) => n,
        Ok(Err(e)) => Err(e)?,
        Err(_) => anyhow::bail!("read timeout"),
    };

    let line = match next_line_break(&buf[..n]) {
        Some(idx) => str::from_utf8(&buf[..idx])?,
        None => anyhow::bail!("too much"),
    };

    let mut parts = line.split_whitespace();

    let method: Method = parts
        .next()
        .ok_or(anyhow::anyhow!("missing method"))
        .and_then(TryInto::try_into)?;

    let path: String = parts
        .next()
        .ok_or(anyhow::anyhow!("missing path"))
        .map(Into::into)?;

    let mut request = Request::new(method, path);

    /*loop {
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
    }*/

    Ok(request)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::Display)]
pub enum Status {
    #[strum(serialize = "200 OK")]
    Ok,

    #[strum(serialize = "200 OK")]
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

    pub async fn write(mut self, stream: &mut TcpStream) -> anyhow::Result<()> {
        match timeout(
            WRITE_TIMEOUT,
            stream.write_all(format!("HTTP/1.1 {}\r\n\r\n", self.status).as_bytes()),
        )
        .await
        {
            Ok(Ok(_)) => (),
            Ok(Err(e)) => Err(e)?,
            Err(_) => anyhow::bail!("write timeout"),
        }

        //tokio::io::copy(&mut self.data, stream).await?;

        Ok(())
    }
}

async fn handle_request(request: Request) -> anyhow::Result<Response> {
    info!("request: {:?}", request);

    let response = Response::new(Status::Ok);

    info!("response: {:?}", response);
    Ok(response)
}

async fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
    let request = read_request(&mut stream).await?;
    let response = handle_request(request).await?;

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
            match handle_connection(stream).await {
                Ok(_) => {}
                Err(e) => {
                    println!("error: {e}");
                }
            }
        });
    }
}
