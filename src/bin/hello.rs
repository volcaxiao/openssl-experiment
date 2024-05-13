use std::error::Error;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Mutex;

use anyhow::{anyhow, Result}; // Add the `anyhow` crate import

use bytes::Bytes;
use futures::FutureExt;
use http_body_util::{BodyExt, Empty, Full};
use http_body_util::combinators::BoxBody;
use hyper::{header, Request, Response, StatusCode};
use hyper::body::{Body, Incoming};
use hyper::client::conn::http1::{Builder, Connection, SendRequest};
use hyper::rt::{Read, Write};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use openssl::ssl::{Ssl, SslAcceptor, SslConnector, SslMethod, SslVerifyMode};
use tokio::net::{TcpListener, TcpStream};
use tokio_openssl::SslStream;
 
pub mod crt;
 
type HttpBody = BoxBody<Bytes, hyper::Error>;
 
#[tokio::main]
pub async fn main() -> Result<()> {
    // This address is localhost
    let addr: SocketAddr = "127.0.0.1:7890".parse().unwrap();
 
    // Bind to the port and listen for incoming TCP connections
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        let (tcp_stream, addr) = listener.accept().await?;
        let _msg = format!("{addr} connected");
        //dbg!(msg);
        tokio::task::spawn(async move {
            let io = TokioIo::new(tcp_stream);
 
            let conn = http1::Builder::new()
                .serve_connection(io, service_fn(handle));
 
            // Don't forget to enable upgrades on the connection.
            let mut conn = conn.with_upgrades();
 
            let conn = Pin::new(&mut conn);
            if let Err(err) = conn.await {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
 
fn get_host_port(host_name: &str) -> (&str, u16) {
    match host_name.find(":") {
        None => {
            (host_name, 80)
        }
        Some(i) => {
            (&host_name[0..i], *&host_name[i + 1..].parse().unwrap_or(80))
        }
    }
}
 
fn not_found_host() -> Response<HttpBody> {
    Response::builder().status(404).body(full("not found host")).unwrap()
}
 
/// Our server HTTP handler to initiate HTTP upgrades.
async fn handle(mut req: Request<Incoming>) -> Result<Response<HttpBody>> {
    if req.method() != hyper::Method::CONNECT {
        let (host, port) = match req.headers().get(header::HOST) {
            None => {
                return Ok(not_found_host());
            }
            Some(h) => { get_host_port(h.to_str()?) }
        };
 
        let stream = TcpStream::connect((host, port)).await?;
        let io = TokioIo::new(stream);
 
        let (mut sender, conn) = Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(io)
            .await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });
        let (req, resp) = intercept_request(req);
        return match resp {
            None => {
                let resp = sender.send_request(req).await?;
                Ok(resp.map(|b| b.boxed()))
            }
            Some(resp) => {
                Ok(resp)
            }
        };
    }
 
    let res = Response::new(empty());
    // handle https
    tokio::task::spawn(async move {
        match hyper::upgrade::on(&mut req).await {
            Ok(upgraded) => {
                if let Some(host) = req.uri().host() {
                    if let Err(e) = server_upgraded_https(host, upgraded).await {
                        let error_msg = format!("server io error: {}", e);
                        dbg!(error_msg);
                    };
                }
            }
            Err(e) => eprintln!("upgrade error: {}", e),
        }
    });
 
    Ok(res)
}
 
 
/// https upgraded
async fn server_upgraded_https(host: &str, upgraded: Upgraded) -> Result<()> {
    let upgraded = TokioIo::new(upgraded);
    // we have an upgraded connection that we can read and
    // write on directly.
    //
    let tls_acceptor = get_tls_acceptor(host);
    let ssl = Ssl::new(tls_acceptor.context())?;
    let mut tls_stream = SslStream::new(ssl, upgraded)?;
    if let Err(err) = SslStream::accept(Pin::new(&mut tls_stream)).await {
        return Err(anyhow!("error during tls handshake connection from : {}", err));
    }
    let stream = TokioIo::new(tls_stream);
 
    let (sender, conn) = https_remote_connect(host, 443).await?;
    tokio::spawn(async move {
        if let Err(err) = conn.await {
            let err_msg = format!("Connection failed: {:?}", err);
            dbg!(err_msg);
        }
    });
    let wrap_sender = Mutex::new(sender);
    if let Err(err) = http1::Builder::new()
        .serve_connection(stream, service_fn(|req| {
            let (req, resp) = intercept_request(req);
            async {
                match resp {
                    None => {
                        let remote_resp = wrap_sender.lock().unwrap().send_request(req);
                        match remote_resp.await {
                            Ok(resp) => {
                                Ok::<_, hyper::Error>(intercept_response(resp))
                            }
                            Err(err) => {
                                let resp = Response::builder()
                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                    .header(header::CONTENT_TYPE, "text/plain")
                                    .body(full(err.to_string())).unwrap();
                                Ok::<_, hyper::Error>(resp)
                            }
                        }
                    }
                    Some(resp) => {
                        Ok::<_, hyper::Error>(resp)
                    }
                }
            }
        })).await {
        println!("Error serving connection: {:?}", err);
    }
 
    Ok(())
}
 
 
fn empty() -> HttpBody {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
 
fn full<T: Into<Bytes>>(chunk: T) -> HttpBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
 
/// Certificate not cached
fn get_tls_acceptor(host: &str) -> SslAcceptor {
    let mut tls_builder = SslAcceptor::mozilla_modern_v5(SslMethod::tls()).unwrap();
 
    let (crt, pri_key) = crt::get_crt_key(host);
 
    tls_builder.set_certificate(&crt).unwrap();
 
    tls_builder.set_private_key(&pri_key).unwrap();
 
    tls_builder.check_private_key().unwrap();
 
    let tls_acceptor = tls_builder.build();
    tls_acceptor
}
 
 
async fn https_remote_connect<B>(host: &str, port: u16) -> Result<(SendRequest<B>, Connection<TokioIo<SslStream<TcpStream>>, B>)>
    where
        B: Body + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn Error + Send + Sync>>, {
    let addr = format!("{}:{}", host, port);
    let tcp_stream = TcpStream::connect(addr).await?;
    let mut builder = SslConnector::builder(SslMethod::tls_client())?;
    builder.set_verify(SslVerifyMode::NONE);
    let connector = builder.build();
    let ssl = Ssl::new(connector.context())?;
    let mut tls_stream = SslStream::new(ssl, tcp_stream)?;
    if let Err(err) = SslStream::connect(Pin::new(&mut tls_stream)).await {
        return Err(anyhow!("error during tls handshake connection from : {}", err));
    }
    let io = TokioIo::new(tls_stream);
    Ok(hyper::client::conn::http1::handshake(io).await?)
}
 
/// Intercept local requests
fn intercept_request(mut request: Request<Incoming>) -> (Request<HttpBody>, Option<Response<HttpBody>>) {
    dbg!(request.uri().to_string());
    request.headers_mut().remove("Accept-Encoding");
    let req = request.map(|b| b.boxed());
 
    if let Some(Ok(host)) = req.headers().get(header::HOST).map(|h| h.to_str()) {
        if host.contains("127.0.0.1:7890")
            || host.contains("localhost:7890")
            || host.contains("baidu") {
            let resp = Response::builder()
                .header(header::CONTENT_TYPE, "text/plain")
                .body(full("Proxylea Server Power By Rust & Hyper\n"));
            return (req, Some(resp.unwrap()));
        }
    }
    (req, None)
}
 
/// Intercept remote responses
fn intercept_response(mut response: Response<Incoming>) -> Response<HttpBody> {
    dbg!({ format!("{:?}", response.headers()) });
    response.headers_mut().insert("proxy-server", "Proxylea".parse().unwrap());
    //let (parts,incoming)=resp.into_parts();
    let resp = response.map(|b| b.map_frame(|frame| {
        if let Some(bytes) = frame.data_ref() {
            //
        }
        frame
    }).boxed());
    resp
}
 
 