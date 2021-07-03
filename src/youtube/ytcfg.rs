use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct YtCfg {
    pub player_js_url: String,
    pub innertube_api_key: String,
}
