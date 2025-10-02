use serde::Deserialize;

#[derive(Deserialize)]
pub struct Recommendations {
    pub total: u32,
}

#[derive(Deserialize)]
pub struct Platforms {
    pub windows: bool,
    pub mac: bool,
    pub linux: bool,
}

#[derive(Deserialize)]
pub struct ReleaseDate {
    pub date: String,
}

#[derive(Deserialize)]
pub struct Price {
   //pub currency: String,
   pub initial: u32,
   //#[serde(rename = "final")]
   //pub final_price: u32,
}

#[derive(Deserialize, Default)]
pub struct Achievements {
    pub total: u32,
}

#[derive(Deserialize)]
pub struct Category {
   pub id: u32,
   pub description: String,
}

#[derive(Deserialize)]
pub struct AppData {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub steam_appid: u32,
    pub name: String,
    pub controller_support: Option<String>,
    pub price_overview: Option<Price>,
    pub release_date: Option<ReleaseDate>,
    pub header_image: Option<String>,
    pub platforms: Option<Platforms>,
    #[serde(default)]
    pub achievements: Achievements,
    pub categories: Option<Vec<Category>>,
    pub dlc: Option<Vec<u32>>,
    pub recommendations: Option<Recommendations>,
}

#[derive(Deserialize)]
pub struct App {
    pub success: bool,
    pub data: Option<AppData>,
}

#[derive(Deserialize)]
pub struct AppInfo {
    pub appid: u32,
}

#[derive(Deserialize)]
pub struct AppList {
    pub apps: Vec<AppInfo>,
}

#[derive(Deserialize)]
pub struct SteamApps {
    pub applist: AppList,
}
