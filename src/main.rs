use serde::{Deserialize};
use reqwest;
use std::collections::HashMap;
use rusqlite::{Connection};
use std::process::exit;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let appsurl: String = String::from(
        "https://api.steampowered.com/ISteamApps/GetAppList/v2/"
    );

    let steamappsinfo: SteamApps = reqwest::get(appsurl)
        .await?
        .json()
        .await?;

    let mut steamapps: HashMap<u32, App> = HashMap::new();
    let baseurl: String = String::from(
        "https://store.steampowered.com/api/appdetails?appids="
    );
    for app in steamappsinfo.applist.apps.iter().take(5) {
        let url = baseurl.clone() + &app.appid.to_string();
        let steamapp_map: HashMap<u32, App> = reqwest::get(&url)
            .await?
            .json()
            .await?;
        if let Some((appid, steamapp)) = steamapp_map.into_iter().next() {
            steamapps.insert(appid, steamapp);
        }
    }
    println!("Entries: {}", &steamappsinfo.applist.apps.len());
    /*
    let db = Connection::open("test.db")?;
    if let Err(e) = initialize_schema(&db) {
        eprintln!("Database failed to open: {e}.");
        exit(1);
    };
    */

    Ok(())
}

#[derive(Deserialize)]
struct Platforms {
    windows: bool,
    mac: bool,
    linux: bool,
}

#[derive(Deserialize)]
struct ReleaseDate {
    date: String,
}

#[derive(Deserialize)]
struct Price {
    currency: String,
    initial: u32,
    #[serde(rename = "final")]
    final_price: u32,
}

#[derive(Deserialize, Default)]
struct Achievements {
    total: u32,
}

#[derive(Deserialize)]
struct Category {
    id: u32,
    description: String,
}

#[derive(Deserialize)]
struct AppData {
    #[serde(rename = "type")]
    kind: Option<String>,
    steam_appid: u32,
    name: String,
    controller_support: Option<String>,
    price: Option<Price>,
    release_date: Option<ReleaseDate>,
    header_image: Option<String>,
    platforms: Option<Platforms>,
    #[serde(default)]
    achievements: Achievements,
    categories: Option<Vec<Category>>,
}

#[derive(Deserialize)]
struct App {
    success: bool,
    data: Option<AppData>,
}

#[derive(Deserialize)]
struct AppInfo {
    appid: u32,
}

#[derive(Deserialize)]
struct AppList {
    apps: Vec<AppInfo>,
}

#[derive(Deserialize)]
struct SteamApps {
    applist: AppList,
}

fn initialize_schema(db: &Connection) -> Result<(), rusqlite::Error> {
    db.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS games (

        );
        CREATE TABLE IF NOT EXISTS categories (

        );
        CREATE TABLE IF NOT EXISTS game_categories (

        );
        "#
    )
}
