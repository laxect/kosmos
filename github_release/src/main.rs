use async_std::task;

mod postamt;
mod release;
mod target;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let postamt_task = task::spawn(async {
        postamt::listen().await.unwrap();
    });
    let update_task = task::spawn(async move {
        let min_5 = std::time::Duration::from_secs(5 * 60);
        loop {
            if let Err(e) = target::update().await {
                eprintln!("Update error: {}", e);
            }
            task::sleep(min_5).await;
        }
    });
    postamt_task.await;
    update_task.await;
    Ok(())
}
