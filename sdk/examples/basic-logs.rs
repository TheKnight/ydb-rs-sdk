use ydb::{ClientBuilder, YdbResult};

#[tokio::main]
async fn main() -> YdbResult<()> {
    // very verbose logs
    tracing_subscriber::fmt()
        // enable everything
        .with_max_level(tracing::Level::TRACE)
        // sets this to be the default, global collector for this application.
        .init();

    let client = ClientBuilder::from_str("grpc://localhost:2136?database=local")?.client()?;
    client.wait().await?;
    println!("done");
    return Ok(());
}
