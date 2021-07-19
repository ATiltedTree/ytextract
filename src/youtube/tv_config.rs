use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TvConfig {
    pub web_player_context_config: WebPlayerContextConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct WebPlayerContextConfig {
    pub web_player_context_config_id_living_room_watch: Config,
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    js_url: String,
    pub innertube_api_key: String,
}

impl Config {
    pub fn js_url(&self) -> String {
        std::iter::once("https://www.youtube.com")
            .chain(self.js_url.split('/').take(4))
            .chain(["player_ias.vflset", "en_US", "base.js"])
            .collect::<Vec<_>>()
            .join("/")
    }
}
