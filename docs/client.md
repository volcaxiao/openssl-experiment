### Rust Client 实现

这里我选择rust实现client端，使用了rust的第三方包`hyper`以及`openssl`用于发起连接和验证SSL请求。

下面是具体代码：

```rust
use http_body_util::Empty;
use hyper::body::{Bytes, Incoming};
use hyper::Request;
use hyper::Response;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use hyper_util::rt::TokioIo;
use openssl::ssl::SslConnector;
use openssl::ssl::SslFiletype;
use openssl::ssl::SslMethod;
use tokio::net::TcpStream;
use tokio_openssl::SslStream;
#[allow(unused)]
async fn get(
    url: &hyper::Uri,
) -> Result<Response<Incoming>, Box<dyn std::error::Error + Send + Sync>> {
    let host = url.host().expect("[GET] url has no host");
    let port = url.port_u16().unwrap_or(80);
    let address = format!("{}:{}", host, port);
    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);
    let authority = url.authority().unwrap().clone();
    let client =
        Client::builder(TokioExecutor::new()).build::<_, Empty<Bytes>>(HttpsConnector::new());
    let res = client.get(url.clone()).await?;
    Ok(res)
}

#[allow(unused)]
async fn delete(
    url: &hyper::Uri,
) -> Result<Response<Incoming>, Box<dyn std::error::Error + Send + Sync>> {
    let host = url.host().expect("[DELETE] url has no host");
    let port = url.port_u16().unwrap_or(80);
    let address = format!("{}:{}", host, port);
    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);
    let authority = url.authority().unwrap().clone();
    let (mut sender, _conn) = hyper::client::conn::http1::handshake(io).await?;
    let req = Request::builder()
        .method("DELETE")
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;

    let res = sender.send_request(req).await?;
    Ok(res)
}

async fn post(
    url: &hyper::Uri,
    body: Bytes,
) -> Result<Response<Incoming>, Box<dyn std::error::Error + Send + Sync>> {
    // 解析 URL 来获取主机名和端口
    let host = url.host().expect("[POST] url has no host");
    let port = url.port_u16().unwrap_or(80);
    let address = format!("{}:{}", host, port);

    // 创建 SSL/TLS 连接器
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.set_ca_file("./SSL/Cert/ca_cert.pem").unwrap(); // 设置 CA 证书
    builder
        .set_private_key_file("./SSL/Cert/ca.key", SslFiletype::PEM)
        .unwrap(); // 设置私钥
    builder
        .set_certificate_chain_file("./SSL/Cert/ca.crt")
        .unwrap(); // 设置证书链
    let connector = builder.build();

    // 连接到服务器
    let stream = TcpStream::connect(address).await?;
    let tls_stream = connector.connect(, stream)
    // 其余的代码...
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let res = get(&"https://baidu.com".parse().unwrap()).await?;
    println!("{:?}", res);
    Ok(())
}
```





### XSS攻击请求

通过将包含恶意代码的文本提交到网站中给所有用户展示的地方来达到XSS攻击的目的，如果在用户提交数据时未对数据进行检验，提交中包含的恶意代码就会嵌入在展示给用户的网页中被执行。例如在这次示例中尝试将一个`<img>`的元素嵌入网页中

```html
<img src=\"invalid-url\" onerror=\"alert(\'You have been XSS attacked\')\">
```

需要注意当`<img>`元素中无法访问`src`属性指定的数据时，就会执行`onerror`指定函数，从而达到了XSS攻击的目的。

```rust	
use reqwest::Certificate;
use reqwest::Client;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let bytes = tokio::fs::read(Path::new("./SSL/Cert/ca_cert.pem")).await?;
    let client = Client::builder()
        .add_root_certificate(reqwest::Certificate::from_pem(&bytes)?).danger_accept_invalid_certs(true)
        .build()?;
    let response = client
        .get("https://127.0.0.1:8443/XSS/unsafe")
        .send()
        .await?;
    let response = client.post("https://127.0.0.1:8443/XSS/unsafe").body("<img src=\"invalid-url\" onerror=\"alert(\'You have been XSS attacked\')\">").send().await?;
    println!("{}", response.text().await?);
    Ok(())
}

```

这里主要使用了rust的第三方库`reqwest`用于发送POST请求，并且添加了之前创建的SSL证书,用于验证。

