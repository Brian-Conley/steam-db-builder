mod types;
use types::*;
use reqwest::Client;
use std::collections::HashMap;
use rusqlite::{Connection};
use std::process::exit;
use std::path::Path;
use serde_json;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Open the database
    let db = Connection::open("steam_games.db")?;
    initialize_schema(&db);

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

    // Load progress vector
    let progress_vector_filename: String = String::from("progress_vector.json");
    let mut progress_vector = load_vector(&progress_vector_filename);

    let total_apps: usize = steamappsinfo.applist.apps.len();
    let mut current_apps: usize = 0;
    let mut requests_made: u32 = 0;

    // Parse every app
    println!("Parsing {} apps.", total_apps);
    for steamapp in steamappsinfo.applist.apps.iter() {

        // Check batch progress, sleep when batch is complete
        if requests_made >= 190 {
            println!("End of batch, sleeping for 5 minutes.");
            save_vector(&progress_vector, &progress_vector_filename);
            sleep(Duration::from_secs(5 * 60)).await;
            requests_made = 0;
        }

        current_apps += 1;
        let appid = steamapp.appid;
        println!("({} / {}) {}", current_apps, total_apps, appid);

        // Check if the appid was already processed
        if progress_vector.contains(&appid) {
            println!("Appid already processed, skipping");
            continue;
        }

        // Fetch the app from the API
        requests_made += 1;
        let app = match get_app(&client, appid).await? {
            Some(app) => app,
            None => {
                println!("Unsuccessful response, skipping.");
                progress_vector.push(appid);
                continue;
            }
        };

        // Push mark app id and process the app data
        progress_vector.push(appid);
        if let Some(data) = app.data {
            println!("Successfully fetched: {}", data.name);

            // Check if the app is actually a game
            match data.kind {
                Some(kind) => {
                    if kind != "game" {
                        continue;
                    }
                },
                None => {
                    continue;
                }
            };

            // Remove all DLC from the pool
            if let Some(dlc) = data.dlc {
                for id in dlc {
                    progress_vector.push(id);
                }
            }


            // Initialize all variables for the db
            let appid: u32 = data.steam_appid;
            let name: String = data.name;
            let controller_support = match data.controller_support {
                Some(sup) => match sup.as_str() {
                    "none" => 0,
                    "partial" => 1,
                    "full" => 2,
                    _ => 3,
                }
                None => 3,
            };
            let price: Option<u32> = match data.price_overview {
                Some(price) => Some(price.initial),
                None => None
            };
            let release_date: Option<String> = match data.release_date {
                Some(date) => Some(date.date),
                None => None,
            };
            let header_image: Option<String> = match data.header_image {
                Some(url) => Some(url),
                None => None,
            };
            let total_recommendations: Option<u32> = match data.recommendations {
                Some(rec) => Some(rec.total),
                None => None,
            };
            let supported_platforms: [Option<bool>; 3] = match data.platforms {
                Some(platforms) => [Some(platforms.windows), Some(platforms.mac), Some(platforms.linux)],
                None => [None, None, None]
            };
            let achievements: bool = match data.achievements.total {
                0 => false,
                _ => true,
            };

            db.execute(
                r#"
                INSERT OR IGNORE INTO games (
                    appid,
                    name,
                    controller_support,
                    has_achievements,
                    supports_windows,
                    supports_mac,
                    supports_linux,
                    price,
                    total_recommendations,
                    release_date,
                    header_image
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ? ,? ,?, ?)
                "#,
                rusqlite::params![
                    appid,
                    name,
                    controller_support,
                    achievements,
                    supported_platforms[0],
                    supported_platforms[1],
                    supported_platforms[2],
                    price,
                    total_recommendations,
                    release_date,
                    header_image,
                ],
            )?;

            // Process each category of the game
            if let Some(v) = data.categories {
                for category in v {
                    db.execute(
                        r#"
                        INSERT OR IGNORE INTO categories(
                            id,
                            name
                        ) VALUES (?, ?)
                        "#,
                        rusqlite::params![
                            category.id,
                            category.description,
                        ],
                    )?;

                    db.execute(
                        r#"
                        INSERT OR IGNORE INTO game_categories(
                            appid,
                            categoryid
                        ) VALUES (?, ?)
                        "#,
                        rusqlite::params![
                            appid,
                            category.id,
                        ]
                    )?;
                }
            }
        }
    }

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

fn load_vector(file: &String) -> Vec<u32> {

    let path = Path::new(file);
    if path.exists() {
        let data = match std::fs::read_to_string(path) {
            Ok(str) => str,
            Err(e) => {
                eprintln!("Error loading progress vector: {}", e);
                exit(1);
            }
        };

        serde_json::from_str(&data).unwrap_or_else(|err| {
            eprintln!("Failed to parse JSON in {}: {}", path.display(), err);
            exit(1);
        })

    } else {
        Vec::new()
    }
}

fn save_vector(v: &Vec<u32>, file: &String) {

    let path = Path::new(file);
    let data = serde_json::to_string(v).unwrap_or_else(|err| {
        eprintln!("Failed to serialize progress vector: {}", err);
        exit(1);
    });

    std::fs::write(path, data).unwrap_or_else(|err| {
        eprintln!("Failed to write progress vector to {}: {}", path.display(), err);
        exit(1);
    });
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
