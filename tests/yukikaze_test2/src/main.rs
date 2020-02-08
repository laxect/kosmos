use async_std::prelude::*;
use kosmos::{client::UnixClient, utils::*};
use yukikaze::Ask;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let mut client = UnixClient::new("test".to_owned());
    let name = client.regist().await?;
    println!("name - {}", name);
    let listener = client.listen().await?;
    let mut incoming = listener.incoming();
    for stream in incoming.next().await {
        let mut stream = stream?;
        let len = stream.get_len().await?;
        if len == 0 {
            return Ok(());
        }
        let obj: Ask = stream.get_obj(len).await?;
        println!("{:?}", obj);
        let exit = [0u8; 4];
        stream.write(&exit);
        return Ok(());
    }
    Ok(())
}
