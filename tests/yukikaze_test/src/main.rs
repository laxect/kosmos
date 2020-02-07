use async_std::prelude::*;
use kosmos::{client::UnixClient, utils::*};
use yukikaze::Request;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let client = UnixClient::new("test".to_owned());
    let mut stream = client.connect("yukikaze").await?;
    let req = Request::post("test".to_owned(), "test".to_owned());
    let req = req.package()?;
    stream.write(&req).await?;
    Ok(())
}
