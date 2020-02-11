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
    let update = task::spawn(async {
        let ten_min = std::time::Duration::from_secs(600);
        loop {
            if let Err(e) = parser::fetch_and_parse().await {
                eprintln!("update - {}", e);
            }
            task::sleep(ten_min).await;
        }
    });
    if let Err(e) = postamt.await {
        eprintln!("postamt - {}", e);
    }
    update.await;
    Ok(())
}
