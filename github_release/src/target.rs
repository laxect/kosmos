use once_cell::sync as cell;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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

const BEHOLDER: [u8; 0] = [0u8; 0];

fn add(target: &Target) -> anyhow::Result<()> {
    let bin_target = bincode::serialize(target)?;
    STORE.insert(bin_target, &BEHOLDER)?;
    Ok(())
}

fn remove(target: &Target) -> anyhow::Result<()> {
    let bin_target = bincode::serialize(target)?;
    STORE.remove(bin_target)?;
    Ok(())
}

pub(crate) fn list() -> anyhow::Result<Vec<Target>> {
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
