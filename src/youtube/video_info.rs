use serde::{Deserialize, Serialize};

static URL: &str = "https://youtube.com/get_video_info";

#[derive(Debug, Deserialize)]
pub struct VideoInfo {
    player_response: String,
}

#[serde_with::serde_as]
#[derive(Serialize, Debug)]
struct RequestParameters {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    video_id: crate::video::Id,
    el: String,
    eurl: String,
    hl: String,
    html5: usize,
    c: String,
    cver: String,
}

impl VideoInfo {
    pub async fn from_id(client: &reqwest::Client, id: crate::video::Id) -> crate::Result<Self> {
        // I love YouTube: https://github.com/ytdl-org/youtube-dl/issues/29333#issuecomment-864049544
        let parms = RequestParameters {
            video_id: id,
            el: String::from("embedded"),
            eurl: format!("https://youtube.googleapis.com/v/{}", id),
            hl: String::from("en"),
            html5: 1,
            c: String::from("TVHTML5"),
            cver: String::from("6.20180913"),
        };

        let response = client
            .get(URL)
            .query(&parms)
            .send()
            .await?
            .error_for_status()?;

        Ok(serde_urlencoded::from_str(&response.text().await?)
            .expect("VideoInfo response was invalid urlencoded"))
    }

    pub(crate) fn player_response(&self) -> crate::youtube::player_response::PlayerResponse {
        serde_json::from_str(&self.player_response).expect("player_response was not valid json")
    }
}
