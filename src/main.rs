mod types;
use types::*;
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

    // Prepare some variables for parsing and to track progress
    let total_apps: usize = steamappsinfo.applist.apps.len();
    let mut current_apps: usize = 0;
    //let mut steamapps: HashMap<u32, App> = HashMap::new();
    let baseurl: String = String::from(
        "https://store.steampowered.com/api/appdetails?appids="
    );

    println!("Parsing {} apps.", total_apps);
    // Rewrite this block to handle all null cases, check http response code, and do batching
    /*
    for app in steamappsinfo.applist.apps.iter().skip(50000).take(1000) {
        let url = baseurl.clone() + &app.appid.to_string();
        let steamapp_map: Option<HashMap<u32, App>> = reqwest::get(&url)
            .await?
            .json()
            .await?;

        let steamapp_map = match steamapp_map {
            Some(map) => map,
            None => {
                println!("Response null: Skipping.");
                continue;
            }
        };

        if let Some((appid, steamapp)) = steamapp_map.into_iter().next() {
            //steamapps.insert(appid, steamapp);
            current_apps += 1;
            println!("({} / {}) {}:", current_apps, total_apps, appid);
            if !steamapp.success {
                println!("Failed to retrieve. Skipping.");
                continue;
            }
            if let Some(ref data) = steamapp.data {
                println!(
                    r#"appid={}, name={}, type={}, controller_support={:?},
price={:?}, release_date={:?}, windows={},
mac={}, linux={}, achievements={}"#,
                    data.steam_appid,
                    data.name,
                    data.kind.clone().unwrap_or_else(|| "unknown".to_string()),
                    data.controller_support,
                    data.price.as_ref().map(|p| format!("{} {}", p.final_price, p.currency)),
                    data.release_date.as_ref().map(|rd| rd.date.clone()),
                    data.platforms.as_ref().map_or(false, |p| p.windows),
                    data.platforms.as_ref().map_or(false, |p| p.mac),
                    data.platforms.as_ref().map_or(false, |p| p.linux),
                    data.achievements.total
                );
            }
        }
    }
    */

    /*
    let db = Connection::open("test.db")?;
    if let Err(e) = initialize_schema(&db) {
        eprintln!("Database failed to open: {e}.");
        exit(1);
    };
    */

    Ok(())
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
