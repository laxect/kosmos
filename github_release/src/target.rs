use crate::{postamt, release};
use async_std::task;
use once_cell::sync as cell;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Target(String, String);

impl Target {
    pub(crate) fn get_user(&self) -> &str {
        &self.0
    }

    pub(crate) fn get_repo(&self) -> &str {
        &self.1
    }

    fn new<R: ToString, U: ToString>(user: U, repo: R) -> Self {
        Self(user.to_string(), repo.to_string())
    }
}

impl<T> From<T> for Target
where
    T: Into<String>,
{
    fn from(item: T) -> Target {
        let item = item.into();
        if item.contains('/') {
            let mut token = item.splitn(2, '/');
            let user = token.next().unwrap();
            let repo = token.next().unwrap();
            Target::new(user, repo)
        } else {
            Target(String::new(), item)
        }
    }
}

impl ToString for Target {
    fn to_string(&self) -> String {
        format!("{}/{}", self.0, self.1)
    }
}

static STORE: cell::Lazy<sled::Db> = cell::Lazy::new(|| sled::open("/tmp/kosmos/db/github_release").unwrap());

type Update = Option<String>;

fn add(target: &Target) -> anyhow::Result<()> {
    let bin_target = bincode::serialize(target)?;
    let mark: Update = None;
    let bin_mark = bincode::serialize(&mark)?;
    STORE.insert(bin_target, bin_mark)?;
    Ok(())
}

fn remove(target: &Target) -> anyhow::Result<()> {
    let bin_target = bincode::serialize(target)?;
    STORE.remove(bin_target)?;
    Ok(())
}

fn check_and_update(target: &Target, new_ver: &str) -> anyhow::Result<bool> {
    let new_ver = Some(new_ver.to_owned());
    let bin_target = bincode::serialize(&target)?;
    let bin_mark = bincode::serialize(&new_ver)?;
    if let Some(bin_old_ver) = STORE.insert(bin_target, bin_mark)? {
        let old_ver: Update = bincode::deserialize(&bin_old_ver)?;
        if old_ver == new_ver {
            return Ok(false);
        }
    } else {
        return Ok(true);
    }
    Ok(true)
}

fn list() -> anyhow::Result<Vec<Target>> {
    let mut res = Vec::new();
    for key in STORE.iter().keys() {
        let key = key?;
        let key = bincode::deserialize(&key)?;
        res.push(key);
    }
    Ok(res)
}

fn format_list() -> anyhow::Result<String> {
    let mut fmt_str = String::new();
    let ts = list()?;
    if ts.is_empty() {
        return Ok(String::from("  None."));
    }
    ts.iter()
        .for_each(|x| fmt_str.push_str(&format!("  - {}\n", x.to_string())));
    Ok(fmt_str)
}

pub(crate) fn cmd<T: Into<String>>(input: T) -> anyhow::Result<String> {
    let input = input.into();
    let mut args = input.split_whitespace();
    let mut resp = "No command gived.".to_owned();
    if let Some(sub) = args.next() {
        resp = match sub {
            "add" => {
                for arg in args {
                    let t = Target::from(arg);
                    add(&t)?;
                }
                format!("Add success.\n{}", format_list()?)
            }
            "remove" => {
                for arg in args {
                    let t = Target::from(arg);
                    remove(&t)?;
                }
                format!("Remove success.\n{}", format_list()?)
            }
            "list" => format!("List target.\n{}", format_list()?),
            _ => "Illeagl command".to_owned(),
        }
    }
    Ok(resp)
}

async fn update_handle(target: Target) -> anyhow::Result<()> {
    let release = release::get(target.clone()).await?;
    let new_ver = release.tag_name.clone();
    println!("{} - {}", target.to_string(), new_ver);
    if check_and_update(&target, &new_ver)? {
        let name = target.get_repo();
        println!("post");
        postamt::post(name, &new_ver).await?;
    }
    Ok(())
}

pub(crate) async fn update() -> anyhow::Result<()> {
    let ts = list()?;
    for t in ts.into_iter() {
        task::spawn(async move {
            if let Err(e) = update_handle(t).await {
                eprintln!("Error: {}", e);
            }
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        let target = Target::from("test/test");
        add(&target)?;
        let first = check_and_update(&target, "0.1.0")?;
        assert!(first);
        let second = check_and_update(&target, "0.1.0")?;
        assert!(!second);
        let third = check_and_update(&target, "0.2.0")?;
        assert!(third);
        remove(&target)?;
        Ok(())
    }
}
