mod release;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let r: release::Release = surf::get("https://api.github.com/repos/rust-analyzer/rust-analyzer/releases/latest")
        .recv_json()
        .await
        .unwrap();
    print!("{:?}", r);
    Ok(())
}
