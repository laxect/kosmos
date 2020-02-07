use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Request {
    Post { msg: String, node: String },
}

impl Request {
    pub fn post(msg: String, node: String) -> Request {
        Request::Post { msg, node }
    }
}

impl Into<String> for Request {
    fn into(self) -> String {
        match self {
            Request::Post { msg, node } => format!("{}\n\n------{} - 雪風Dタイプ", msg, node),
        }
    }
}
