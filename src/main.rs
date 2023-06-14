use clap::Parser;
use reqwest::blocking::ClientBuilder;
use std::io::BufRead;

const NETWORK_ID: &str = "9126368";

#[derive(Parser)]
struct Cli {
    #[arg(short)]
    cookie: Option<String>,
    #[arg(long)]
    login: Option<String>,
    #[arg(long)]
    cmd: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct Device {
    pub display_name: String,
    pub hostname: String,
    pub connected: bool,
}

fn main() -> std::io::Result<()> {
    let base_url = "https://api-user.e2ro.com/2.2";
    let client = ClientBuilder::new().cookie_store(true).build().unwrap();

    let args = Cli::parse();

    if let Some(cookie) = args.cookie {
        let response = client
            .post(format!("{base_url}/login/refresh"))
            .header(reqwest::header::COOKIE, format!("s={cookie}"))
            .send()
            .unwrap()
            .text()
            .unwrap();
        let response_json = serde_json::from_str::<serde_json::Value>(&response).unwrap();
        let cookie = response_json["data"]["user_token"]
            .as_str()
            .unwrap()
            .to_owned();
        let cmd_str = args.cmd.as_ref().map(|s| s.as_str());
        match cmd_str {
            Some("devices") => {
                let response = client
                    .get(format!("{base_url}/networks/{NETWORK_ID}/devices"))
                    .header(reqwest::header::COOKIE, format!("s={cookie}"))
                    .send()
                    .unwrap();
                if response.status().is_success() {
                    let response_json =
                        serde_json::from_str::<serde_json::Value>(&response.text().unwrap())
                            .unwrap();
                    let devices = response_json["data"].as_array().unwrap();
                    for device in devices {
                        if let Ok(device) = serde_json::from_value::<Device>(device.clone()) {
                            println!("{device:?}")
                        }
                    }
                } else {
                    println!("Fail: {}", response.status());
                }
            }
            Some(_) => {
                println!("Unsupported");
            }
            None => {
                let response = client
                    .get(format!("{base_url}/account"))
                    .header(reqwest::header::COOKIE, format!("s={cookie}"))
                    .send()
                    .unwrap();
                if response.status().is_success() {
                    println!("{}", response.text().unwrap());
                } else {
                    println!("Fail: {}", response.status());
                }
            }
        }
    } else {
        let response = client
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({ "login": args.login.unwrap() }))
            .send()
            .unwrap()
            .text()
            .unwrap();
        let response_json = serde_json::from_str::<serde_json::Value>(&response).unwrap();
        println!("{response_json}");
        let cookie = response_json["data"]["user_token"]
            .as_str()
            .unwrap()
            .to_owned();
        println!("cookie: {}", &cookie);
        let mut verification_code = String::new();
        let stdin = std::io::stdin();
        stdin.lock().read_line(&mut verification_code).unwrap();
        let response = client
            .post(format!("{base_url}/login/verify"))
            .header(reqwest::header::COOKIE, format!("s={cookie}"))
            .json(&serde_json::json!({ "code": verification_code.trim() }))
            .send()
            .unwrap();
        if response.status().is_success() {
            println!("Success");
        } else {
            println!("Fail: {}", response.status());
        }
    }

    Ok(())
}
