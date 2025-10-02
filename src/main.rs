mod types;
use types::*;
use reqwest::Client;
use std::collections::HashMap;
use rusqlite::{Connection};
use std::process::exit;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Create the web client
    let client = Client::builder()
        .user_agent("student-steam-db-builder")
        .build()?;

    // Build a list of all apps by id
    let appsurl: String = String::from(
        "https://api.steampowered.com/ISteamApps/GetAppList/v2/"
    );
    let steamappsinfo: SteamApps = client.get(appsurl).send()
        .await?
        .json()
        .await?;

    let total_apps: usize = steamappsinfo.applist.apps.len();
    let mut current_apps: usize = 0;

    println!("Parsing {} apps.", total_apps);
    for steamapp in steamappsinfo.applist.apps.iter().skip(50000).take(20) {
        current_apps += 1;
        let appid = steamapp.appid;
        println!("({} / {}) {}", current_apps, total_apps, appid);
        let app = match get_app(&client, appid).await? {
            Some(app) => app,
            None => {
                println!("Invalid response, skipping.");
                continue;
            }
        };
        if !app.success {
            println!("Unsuccessful request, skipping.");
            continue;
        }
        if let Some(data) = app.data {
            println!("Successfully fetched: {}", data.name);
        }
    }

    /*
    let db = Connection::open("test.db")?;
    if let Err(e) = initialize_schema(&db) {
        eprintln!("Database failed to open: {e}.");
        exit(1);
    };
    */

    Ok(())
}

async fn get_app(client: &Client, appid: u32) -> Result<Option<App>, reqwest::Error> {

    // Form the correct url
    let base_url = "https://store.steampowered.com/api/appdetails?appids=";
    let url = format!("{}{}", base_url, appid);

    // Send request
    let resp: HashMap<u32, App> = client
        .get(&url)
        .send()
        .await?
        .json()
        .await?;

    // Check if the request succeeded, return result
    if let Some((_id, app)) = resp.into_iter().next() {
        if app.success {
            Ok(Some(app))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn initialize_schema(db: &Connection) {
    if let Err(e) = db.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS games (
            appid INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            controller_support INTEGER,
            has_achievements BOOLEAN,
            supports_windows BOOLEAN,
            supports_mac BOOLEAN,
            supports_linux BOOLEAN,
            price REAL,
            total_recommendations INTEGER,
            release_date DATETIME,
            header_image TEXT
        );

        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL
        );

        CREATE TABLE IF NOT EXISTS game_categories (
            appid INTEGER,
            categoryid INTEGER,
            PRIMARY KEY (appid, categoryid),
            FOREIGN KEY (appid) REFERENCES games(appid),
            FOREIGN KEY (categoryid) REFERENCES categories(id)
        );

        CREATE INDEX IF NOT EXISTS idx_games_name ON games(name);
        CREATE INDEX IF NOT EXISTS idx_game_categories_appid ON game_categories(appid);
        CREATE INDEX IF NOT EXISTS idx_game_categories_categoryid ON game_categories(categoryid);
        "#
    ) {
        eprintln!("Error while initializing schema: {}", e);
        exit(1);
    } 
}
