use kosmos::server::{link_init, UnixSocketServer};

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    link_init().await?;
    let server = UnixSocketServer::new("kosmos")?;
    server.clear()?;
    server.listen().await?;
    Ok(())
}
