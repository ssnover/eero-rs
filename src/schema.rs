use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Response {
    data: serde_json::Value,
}

impl Response {
    pub fn from_response<T: DeserializeOwned>(self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.data)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct UserToken {
    pub user_token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Device {
    pub display_name: String,
    pub hostname: Option<String>,
    pub connected: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountSummary {
    pub networks: NetworksCollection,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworksCollection {
    pub count: usize,
    pub data: Vec<Network>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Network {
    pub name: String,
    pub url: String,
}
