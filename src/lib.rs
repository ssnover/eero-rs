use reqwest::blocking::ClientBuilder;
use secrecy::{ExposeSecret, Secret};

mod schema;
use schema::*;

const API_BASE_URL: &str = "https://api-user.e2ro.com/2.2";

pub enum LoginMode {
    Email(String),
    PhoneNumber(String),
}

impl ToString for LoginMode {
    fn to_string(&self) -> String {
        match self {
            LoginMode::Email(email) => email.clone(),
            LoginMode::PhoneNumber(number) => number.clone(),
        }
    }
}

pub fn get_user_token(login: LoginMode) -> Result<String, Box<dyn std::error::Error>> {
    let client = ClientBuilder::new().build()?;
    let response = client
        .post(format!("{API_BASE_URL}/login"))
        .json(&serde_json::json!({ "login": login.to_string() }))
        .send()?
        .text()?;
    let user_token = serde_json::from_str::<Response>(&response)?.from_response::<UserToken>()?;
    Ok(user_token.user_token)
}

pub fn confirm_user_token_with_verification_code(
    user_token: &str,
    verification_code: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let client = ClientBuilder::new().cookie_store(true).build()?;
    let response = client
        .post(format!("{API_BASE_URL}/login/verify"))
        .header(reqwest::header::COOKIE, format!("s={user_token}"))
        .json(&serde_json::json!({ "code": verification_code }))
        .send()?;
    if response.status().is_success() {
        println!("{}", response.text().unwrap());
        Ok(true)
    } else {
        log::error!("Fail: {}", response.status());
        Ok(false)
    }
}

pub struct Client {
    inner: reqwest::Client,
    user_token: Secret<String>,
}

impl Client {
    pub fn new(user_token: &str) -> Self {
        Self {
            inner: reqwest::ClientBuilder::new()
                .cookie_store(true)
                .build()
                .unwrap(),
            user_token: Secret::new(user_token.to_owned()),
        }
    }

    async fn refresh_user_token(&mut self) -> Result<(), reqwest::Error> {
        let response = self
            .inner
            .post(format!("{API_BASE_URL}/login/refresh"))
            .header(
                reqwest::header::COOKIE,
                format!("s={}", self.user_token_as_cookie()),
            )
            .send()
            .await?;
        let response = response.text().await?;
        let user_token = serde_json::from_str::<Response>(&response)
            .unwrap()
            .from_response::<UserToken>()
            .unwrap();
        let user_token = user_token.user_token.trim_start_matches("s=");
        self.user_token = Secret::new(user_token.to_owned());
        Ok(())
    }

    fn user_token_as_cookie(&self) -> String {
        format!("s={}", self.user_token.expose_secret())
    }

    async fn get(
        &mut self,
        route: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.refresh_user_token().await?;
        let builder = self
            .inner
            .get(format!("{API_BASE_URL}/{route}"))
            .header(reqwest::header::COOKIE, self.user_token_as_cookie());
        let builder = if let Some(body) = body {
            builder.json(&body)
        } else {
            builder
        };
        builder.send().await
    }

    async fn post(
        &mut self,
        route: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.refresh_user_token().await?;
        self.inner
            .post(format!("{API_BASE_URL}/{route}"))
            .header(reqwest::header::COOKIE, self.user_token_as_cookie())
            .json(&body)
            .send()
            .await
    }

    pub async fn get_account_summary(
        &mut self,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let route = "account";
        let response = self.get(&route, None).await?;
        Ok(serde_json::from_str::<serde_json::Value>(&response.text().await?).unwrap())
    }

    pub async fn get_network_ids(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let acct_summary =
            serde_json::from_value::<AccountSummary>(self.get_account_summary().await?)?;
        let network_ids = acct_summary
            .networks
            .data
            .into_iter()
            .map(|network_data| network_data.url.split('/').last().unwrap().to_owned())
            .collect();
        Ok(network_ids)
    }

    pub async fn get_devices_for_network(
        &mut self,
        network_id: &str,
    ) -> Result<Vec<Device>, Box<dyn std::error::Error>> {
        let route = format!("networks/{network_id}/devices");
        let response = self.get(&route, None).await?;
        if response.status().is_success() {
            let response = response.text().await?;
            let response = serde_json::from_str::<Response>(&response).unwrap();
            let network_elements = response.from_response::<Vec<serde_json::Value>>().unwrap();
            Ok(network_elements
                .into_iter()
                .flat_map(|element| serde_json::from_value::<Device>(element))
                .collect())
        } else {
            Err(Box::new(std::io::Error::from(
                std::io::ErrorKind::Unsupported,
            )))
        }
    }
}
