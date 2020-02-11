use crate::store::Store;
use serde::{Deserialize, Serialize};

const LN_XML_URI: &str = "https://www.lightnovel.us/forum.php?mod=rss&fid=173";

#[derive(Serialize, Deserialize, Clone, Copy)]
pub(crate) enum PageStatus {
    Pending,
    Complete,
}

trait ToMsg<T> {
    fn to_msg(self) -> Result<T, anyhow::Error>;
}

impl<T> ToMsg<T> for Result<T, surf::Exception> {
    fn to_msg(self) -> Result<T, anyhow::Error> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(anyhow::Error::msg(e.to_string())),
        }
    }
}

fn parse_id(link: &str) -> anyhow::Result<u32> {
    let mut link = link.to_string();
    let mut frag = link.split_off(33);
    for _ in 0..9 {
        frag.pop();
    }
    Ok(frag.parse()?)
}

pub(crate) async fn fetch_rss() -> anyhow::Result<()> {
    let rss_origin = surf::get(LN_XML_URI).recv_bytes().await.to_msg()?;
    let rss_parser = rss::Channel::read_from(rss_origin.as_ref())?;
    let keyword = Store::new("keyword")?;
    let watch_list = Store::new("page")?;
    let config = Store::new("config")?;
    let id_key = "update_id".to_owned();
    let mut new_id = 0;
    let id: u32 = config.get(&id_key)?.unwrap_or_default();
    for item in rss_parser.items().into_iter() {
        let title = item.title().unwrap_or_default();
        let link = item.link().unwrap_or_default();
        let this_id = parse_id(link)?;
        if this_id > id {
            if this_id > new_id {
                new_id = this_id;
            }
        } else {
            continue;
        }
        // split title
        let memes = title.split(|s| s == '[' || s == ']' || s == '(' || s == ')' || s == '【' || s == '】' || s == ' ');
        let mut res = false;
        for meme in memes {
            if keyword.contains_key(meme)? {
                res = true;
                break;
            }
        }
        if res {
            let item = PageStatus::Pending;
            watch_list.insert(&link, &item)?;
        }
    }
    config.insert(&id_key, &new_id)?;
    Ok(())
}

fn translate_uri(uri: String) -> anyhow::Result<String> {
    if !uri.contains("php") {
        // doesn't need translate
        return Ok(uri);
    }
    let extra = uri.find("&extra").ok_or(anyhow::Error::msg("can not translate"))?;
    let tid = uri.find("&tid").ok_or(anyhow::Error::msg("can not translate"))?;
    let tid = tid + 5;
    let tid: u32 = uri[tid..extra].parse()?;
    let uri = format!("https://www.lightnovel.us/thread-{}-1-1.html", tid);
    Ok(uri)
}

pub(crate) async fn fetch_index(uri: &str) -> anyhow::Result<()> {
    let index_html = surf::get(uri).recv_string().await.to_msg()?;
    let watch_list = Store::new("page")?;
    let html = scraper::Html::parse_document(&index_html);
    let css_select = "tbody[id^=normalthread] .s.xst";
    let selector = scraper::Selector::parse(css_select).map_err(|_| anyhow::Error::msg("selector parse error"))?;
    for ele in html.select(&selector) {
        let title = ele.text().collect::<Vec<&str>>().concat();
        let link: String = ele.value().attr("href").unwrap_or_default().to_owned();
        let link = translate_uri(link)?;
        if let Some(PageStatus::Pending) = watch_list.get(&link)? {
            // send notification
            println!("{} - {}", &link, &title);
            let new_val = PageStatus::Complete;
            watch_list.insert(&link, &new_val)?;
        }
    }
    Ok(())
}

const LN_REPOST_URI: &str = "https://www.lightnovel.us/forum.php?mod=forumdisplay&fid=173&filter=typeid&typeid=365";
const LN_TRANSLATE_URI: &str = "https://www.lightnovel.us/forum.php?mod=forumdisplay&fid=173&filter=typeid&typeid=369";
const LN_INPUT_URI: &str = "https://www.lightnovel.us/forum.php?mod=forumdisplay&fid=173&filter=typeid&typeid=367";

pub(crate) async fn fetch_and_parse() -> anyhow::Result<()> {
    fetch_rss().await?;
    fetch_index(LN_REPOST_URI).await?;
    fetch_index(LN_TRANSLATE_URI).await?;
    fetch_index(LN_INPUT_URI).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_id_test() -> anyhow::Result<()> {
        let input = "https://www.lightnovel.us/thread-1016638-1-1.html";
        let out = 1016638;
        let id = parse_id(input)?;
        assert_eq!(out, id);
        Ok(())
    }

    #[test]
    fn translate_uri_test() -> anyhow::Result<()> {
        let input = "https://www.lightnovel.us/forum.php?mod=viewthread&tid=1015421&extra=page%3D1%26filter%3Dtypeid%26typeid%3D367";
        let expect_out = "https://www.lightnovel.us/thread-1015421-1-1.html";
        let real_out = translate_uri(input.to_owned())?;
        assert_eq!(real_out, expect_out);
        Ok(())
    }
}
