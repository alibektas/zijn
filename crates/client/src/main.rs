use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client
        .get("http://localhost:7000/auth")
        .send()
        .await?
        .text()
        .await?;

    println!("Response: {}", response);
    Ok(())
}
