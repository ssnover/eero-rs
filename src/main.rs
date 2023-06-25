use clap::Parser;
use eero_client::{confirm_user_token_with_verification_code, get_user_token, LoginMode};
use std::io::BufRead;

const NETWORK_ID: &str = "9126368";

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    cookie: Option<String>,
    #[arg(long)]
    login: Option<String>,
    #[arg(long)]
    cmd: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    if let Some(cookie) = args.cookie {
        let cmd_str = args.cmd.as_ref().map(|s| s.as_str());
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(run_cmd(&cookie, cmd_str))?;
    } else {
        let cookie = get_user_token(LoginMode::Email(args.login.unwrap())).unwrap();
        let mut verification_code = String::new();
        let stdin = std::io::stdin();
        stdin.lock().read_line(&mut verification_code).unwrap();
        if let Ok(true) =
            confirm_user_token_with_verification_code(&cookie, &verification_code.trim())
        {
            println!("Successfully verified user token: {cookie}");
        } else {
            println!("Failed to authenticate");
        }
    }

    Ok(())
}

async fn run_cmd(cookie: &str, cmd: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = eero_client::Client::new(&cookie);
    match cmd {
        Some("devices") => {
            if let Ok(devices) = client.get_devices_for_network(NETWORK_ID).await {
                for device in devices {
                    println!("{device:?}")
                }
            } else {
                println!("Failed to query devices");
            }
        }
        Some(_) => {
            println!("Unsupported");
        }
        None => {
            let summary = client.get_account_summary().await?;
            println!("Summary: {summary}");
        }
    }
    Ok(())
}
