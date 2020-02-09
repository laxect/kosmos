#![feature(async_closure)]
use async_std::task;

mod parser;
mod postamt;
mod request;
mod store;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    // link to yukikaze
    let postamt = task::spawn(async || -> anyhow::Result<()> {
        let mut postamt = postamt::Postamt::default();
        postamt.regist().await?;
        postamt.listen().await?;
        Ok(())
    }());
    if let Err(e) = postamt.await {
        eprintln!("postamt - {}", e);
    }
    Ok(())
}
