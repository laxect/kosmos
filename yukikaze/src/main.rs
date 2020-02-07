#![feature(async_closure)]
mod bot;
mod postamt;

use async_std::task;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let postamt = task::spawn(async || -> anyhow::Result<()> {
        let mut postamt = postamt::Postamt::default();
        postamt.regist().await?;
        postamt.listen().await?;
        Ok(())
    }());
    postamt.await?;
    Ok(())
}
