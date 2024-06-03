use openssl::ssl::{Ssl, SslAcceptor, SslFiletype, SslMethod};
use std::io::{Read, Write};
use std::pin::Pin;
use std::sync::Arc;
use tokio_openssl::SslStream;

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
const CERT_PATH: &str = "./SSL/Cert";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载 SSL 证书和私钥
    let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    acceptor
        .set_private_key_file(format!("{}/ca.key", CERT_PATH), SslFiletype::PEM)
        .unwrap();
    acceptor
        .set_certificate_chain_file(format!("{}/ca.crt", CERT_PATH))
        .unwrap();
    acceptor.check_private_key().unwrap();
    // acceptor.set_min_proto_version(Some(openssl::ssl::SslVersion::TLS1_3)).unwrap();
    let acceptor = Arc::new(acceptor.build());
    // let ssl = Ssl::new(ssl_ctx.as_ref()).unwrap();

    // 监听端口
    let listener = TcpListener::bind("127.0.0.1:8443").await?;

    loop {
        // 处理 TCP 连接
        match listener.accept().await {
            Ok((stream, _)) => {
                let acceptor = acceptor.clone();

                // 异步 tokio::spawn 函数，用于处理客户端连接
                tokio::spawn(async move {
                    // 创建 SSL 上下文
                    let ssl = Ssl::new(acceptor.context()).unwrap();

                    // 创建 SSL 流
                    let mut tls_stream = match SslStream::new(ssl, stream) {
                        Ok(stream) => stream,
                        Err(err) => {
                            eprintln!("Error creating SSL stream: {}", err);
                            return;
                        }
                    };

                    // 接受客户端连接
                    if let Err(error) = SslStream::accept(Pin::new(&mut tls_stream)).await {
                        // 接受连接失败
                        eprintln!("Error accepting connection: {}", error);
                        return;
                    }
                    // 创建 TokioIo 流
                    let stream = TokioIo::new(tls_stream);
                    // 创建 HTTP1 服务
                    let conn = http1::Builder::new().serve_connection(stream, service_fn(handle));
                    // 处理客户端连接
                    if let Err(err) = conn.await {
                        // 处理连接失败
                        eprintln!("Error serving connection: {}", err);
                        return;
                    }
                });
            }
            Err(err) => {
                eprintln!("Error accepting connection: {}", err)
            }
        };
    }
}

// 异步函数，处理请求
async fn handle(
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    // 根据请求方法，调用相应的处理函数
    match *request.method() {
        hyper::Method::GET => handle_get(request).await,
        hyper::Method::POST => handle_post(request).await,
        _ => {
            // 如果请求方法不是GET或POST，返回404错误
            let response = Response::builder()
                .status(hyper::StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("404 Not Found")))
                .unwrap();
            Ok(response)
        }
    }
}

// 读取文件内容
fn read_file(path: &str) -> Result<String, std::io::Error> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

// 处理 Get 请求, 返回对应路径的文件内容
async fn handle_get(
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = request.uri().path();
    // 定义一个基本的 CSP 策略字符串, 允许从当前源加载, 并且只允许内联样式
    let csp_header_value = "default-src 'self'; script-src 'self';";
    let csp_header = HeaderValue::from_str(csp_header_value)
        .expect("Failed to convert CSP string into HeaderValue");
    // 定义一个cookie，供会话劫持的演练
    let cookie = HeaderValue::from_str("session=iamaSess")
        .expect("Failed to convert cookie string into HeaderValue");
    match path {
        "/XSS/safe" => {
            let file_path = format!("{}/XSS/index.html", ROOT_PATH);
            match read_file(file_path.as_str()) {
                Ok(buf) => {
                    // 创建响应
                    let mut response = Response::new(Full::new(Bytes::from(buf)));
                    // 添加 cookie 头
                    response.headers_mut().insert(header::SET_COOKIE, cookie);
                    // 添加 CSP 头
                    response
                        .headers_mut()
                        .insert(header::CONTENT_SECURITY_POLICY, csp_header);
                    Ok(response)
                }
                Err(_) => {
                    let response = Response::new(Full::new(Bytes::from("404 Not Found")));
                    Ok(response)
                }
            }
        }
        "/XSS/unsafe" => {
            let file_path = format!("{}/XSS/index.html", ROOT_PATH);
            match read_file(file_path.as_str()) {
                Ok(buf) => {
                    // 生成响应
                    let mut response = Response::new(Full::new(Bytes::from(buf)));
                    // 添加 cookie 头
                    response.headers_mut().insert(header::SET_COOKIE, cookie);
                    Ok(response)
                }
                Err(_) => {
                    let response = Response::new(Full::new(Bytes::from("404 Not Found")));
                    Ok(response)
                }
            }
        }
        "/XSS/input" => {
            let file_path = format!("{}/XSS/input.txt", ROOT_PATH);
            match read_file(file_path.as_str()) {
                Ok(buf) => {
                    let response = Response::new(Full::new(Bytes::from(buf)));
                    Ok(response)
                }
                Err(_) => {
                    let response = Response::new(Full::new(Bytes::from("404 Not Found")));
                    Ok(response)
                }
            }
        }
        "/XSS/handle.js" => {
            let file_path = format!("{}/{}", ROOT_PATH, path);
            match read_file(file_path.as_str()) {
                Ok(buf) => {
                    let response = Response::new(Full::new(Bytes::from(buf)));
                    Ok(response)
                }
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

fn write_file(path: &str, content: &str) -> Result<(), std::io::Error> {
    let mut file = std::fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

// 处理 Post 请求, 将 返回 OK
async fn handle_post(
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = request.uri().path().to_string();

    let body = request
        .into_body()
        .frame()
        .await
        .unwrap()
        .unwrap()
        .into_data()
        .unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    match path.as_str() {
        "/XSS/safe" => {
            let file_path = format!("{}/XSS/input.txt", ROOT_PATH);
            // 使用 ammonia 库来清理 HTML 内容
            let body = ammonia::clean(&body);
            // 将过滤后输入存入文件
            match write_file(&file_path, &body) {
                Ok(_) => {
                    let response = Response::new(Full::new(Bytes::from("OK")));
                    Ok(response)
                }
                Err(_) => {
                    let response = Response::new(Full::new(Bytes::from("404 Not Found")));
                    Ok(response)
                }
            }
        }
        "/XSS/unsafe" => {
            let file_path = format!("{}/XSS/input.txt", ROOT_PATH);
            // 将输入直接存入文件
            match write_file(&file_path, &body) {
                Ok(_) => {
                    let response = Response::new(Full::new(Bytes::from("OK")));
                    Ok(response)
                }
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
