use serde::{Deserialize};
use reqwest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let appsurl: String = String::from(
        "https://api.steampowered.com/ISteamApps/GetAppList/v2/"
    );

    let steamapps: SteamApps = reqwest::get(appsurl)
        .await?
        .json()
        .await?;

    println!("{:?}", steamapps);

    Ok(())
}

#[derive(Debug, Deserialize)]
struct AppInfo {
    appid: u32,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Applist {
    apps: Vec<AppInfo>,
}

#[derive(Debug, Deserialize)]
struct SteamApps {
    applist: Applist,
}
