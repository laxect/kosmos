use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Ask(pub String);

impl Ask {
    pub fn new(s: String) -> Ask {
        Ask(s)
    }
}

impl Into<String> for Ask {
    fn into(self) -> String {
        self.0
    }
}
