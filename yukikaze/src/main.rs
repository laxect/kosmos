#![feature(async_closure, box_patterns)]
mod bot;
mod postamt;

use async_std::task;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    let postamt = task::spawn(async || -> anyhow::Result<()> {
        let mut postamt = postamt::Postamt::default();
        postamt.regist().await?;
        postamt.listen().await?;
        Ok(())
    }());
    let postamt_incoming = task::spawn(async || -> anyhow::Result<()> {
        let postamt = postamt::Postamt::default();
        let mut bot = None;
        loop {
            bot = postamt
                .incoming(bot)
                .await
                .map_err(|e| {
                    eprintln!("postamt - error: {}", e);
                    e
                })
                .ok();
            let two_sec = std::time::Duration::from_secs(2);
            task::sleep(two_sec).await;
        }
    }());
    postamt.await?;
    postamt_incoming.await?;
    Ok(())
}
