use openssl::ssl::{Ssl, SslAcceptor, SslFiletype, SslMethod};
use tokio_openssl::SslStream;
use std::io::{Read, Write};
use std::pin::Pin;
use std::sync::Arc;

use std::convert::Infallible;

use http::{header, HeaderValue};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

const ROOT_PATH: &str = "./SSL/WebServer";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载 SSL 证书和私钥
    let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    acceptor
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    acceptor.set_certificate_chain_file("cert.pem").unwrap();
    acceptor.check_private_key().unwrap();
    let acceptor = Arc::new(acceptor.build());
    // let ssl = Ssl::new(ssl_ctx.as_ref()).unwrap();

    let listener = TcpListener::bind("127.0.0.1:8443").await?;

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let acceptor = acceptor.clone();
                tokio::spawn(async move {
                    let ssl = Ssl::new(acceptor.context()).unwrap();

                    let mut tls_stream = match SslStream::new(ssl, stream) {
                        Ok(stream) => stream,
                        Err(err) => {
                            eprintln!("Error creating SSL stream: {}", err);
                            return ;
                        }
                    };

                    if let Err(error) = SslStream::accept(Pin::new(&mut tls_stream)).await {
                        eprintln!("Error accepting connection: {}", error);
                        return ;
                    }
                    // let stream = TokioIo::new(stream);
                    let stream = TokioIo::new(tls_stream);
                    let conn = http1::Builder::new().serve_connection(stream, service_fn(handle));
                    if let Err(err) = conn.await {
                        eprintln!("Error serving connection: {}", err);
                        return ;
                    }
                });
            }
            Err(err) => {
                eprintln!("Error accepting connection: {}", err)
            }
        };
    }
}

async fn handle(request: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    match *request.method() {
        hyper::Method::GET => handle_get(request).await,
        hyper::Method::POST => handle_post(request).await,
        _ => {
            let response = Response::builder()
                .status(hyper::StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("404 Not Found")))
                .unwrap();
            Ok(response)
        }
    }
}

fn read_file(path: &str) -> Result<String, std::io::Error> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

// 处理 Get 请求, 返回对应路径的文件内容
async fn handle_get(request: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = request.uri().path();
    match path {
        "/XSS/safe" => {
           // 定义一个基本的 CSP 策略字符串, 允许从当前源加载, 并且只允许内联样式
            let csp_header_value = "default-src 'self'; script-src 'self';";
            let csp_header = HeaderValue::from_str(csp_header_value)
                .expect("Failed to convert CSP string into HeaderValue");

            let file_path = format!("{}/XSS/index.html", ROOT_PATH);
            match read_file(file_path.as_str()) {
                Ok(buf) => {
                    // 创建响应并添加 CSP 头
                    let mut response = Response::new(Full::new(Bytes::from(buf)));
                    response.headers_mut().insert(header::CONTENT_SECURITY_POLICY, csp_header);

                    Ok(response)
                },
                Err(_) => {
                    let response = Response::new(Full::new(Bytes::from("404 Not Found")));
                    Ok(response)
                }
            }
        },
        "/XSS/unsafe" => {
            let file_path = format!("{}/XSS/index.html", ROOT_PATH);
            match read_file(file_path.as_str()) {
                Ok(buf) => {
                    let response = Response::new(Full::new(Bytes::from(buf)));
                    Ok(response)
                },
                Err(_) => {
                    let response = Response::new(Full::new(Bytes::from("404 Not Found")));
                    Ok(response)
                }
            }
        },
        "/XSS/input" => {
            let file_path = format!("{}/XSS/input.txt", ROOT_PATH);
            match read_file(file_path.as_str()) {
                Ok(buf) => {
                    let response = Response::new(Full::new(Bytes::from(buf)));
                    Ok(response)
                },
                Err(_) => {
                    let response = Response::new(Full::new(Bytes::from("404 Not Found")));
                    Ok(response)
                }
            }
        },
        "/XSS/handle.js" => {
            let file_path = format!("{}/{}", ROOT_PATH, path);
            match read_file(file_path.as_str()) {
                Ok(buf) => {
                    let response = Response::new(Full::new(Bytes::from(buf)));
                    Ok(response)
                },
                Err(_) => {
                    let response = Response::new(Full::new(Bytes::from("404 Not Found")));
                    Ok(response)
                }
            }
        },
        _ => {
            let response = Response::new(Full::new(Bytes::from("404 Not Found")));
            Ok(response)
        }
    }
}

fn write_file(path: &str, content: &str) -> Result<(), std::io::Error> {
    let mut file = std::fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

// 处理 Post 请求, 将 返回 OK
async fn handle_post(request: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = request.uri().path().to_string();

    let body = request.into_body().frame().await.unwrap().unwrap().into_data().unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    println!("body: {}", body);
    match path.as_str() {
        "/XSS/safe" => {
            let file_path = format!("{}/XSS/input.txt", ROOT_PATH);
            // 使用 ammonia 库来清理 HTML 内容
            let body = ammonia::clean(&body);
            match write_file(&file_path, &body) {
                Ok(_) => {
                    let response = Response::new(Full::new(Bytes::from("OK")));
                    Ok(response)
                },
                Err(_) => {
                    let response = Response::new(Full::new(Bytes::from("404 Not Found")));
                    Ok(response)
                }
            }
        },
        "/XSS/unsafe" => {
            let file_path = format!("{}/XSS/input.txt", ROOT_PATH);
            match write_file(&file_path, &body) {
                Ok(_) => {
                    let response = Response::new(Full::new(Bytes::from("OK"))); 
                    Ok(response)
                },
                Err(_) => {
                    let response = Response::new(Full::new(Bytes::from("404 Not Found")));
                    Ok(response)
                }
            }
        }
        _ => {
            let response = Response::new(Full::new(Bytes::from("404 Not Found")));
            Ok(response)
        }
    }
}





