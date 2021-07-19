use serde::Deserialize;

pub mod channel;
pub mod continuation;
pub mod playlist;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Result<T> {
    Error { alerts: (Alert,) },
    Ok(T),
}

impl<T> Result<T> {
    pub fn into_std(self) -> crate::Result<T> {
        match self {
            Self::Error { alerts } => {
                assert_eq!(alerts.0.alert_renderer.r#type, "ERROR");

                let text = alerts.0.alert_renderer.text.runs.0.text;
                eprintln!("{}", text);
                match text.as_str() {
                    "The playlist does not exist." | "This channel does not exist." => {
                        Err(crate::Error::Youtube(crate::error::Youtube::NotFound))
                    }
                    "This playlist type is unviewable." => {
                        Err(crate::Error::Youtube(crate::error::Youtube::Unviewable))
                    }
                    e => unreachable!("Unknown Error text: '{}'", e),
                }
            }
            Self::Ok(ok) => Ok(ok),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub alert_renderer: AlertRenderer,
}

#[derive(Debug, Deserialize)]
pub struct AlertRenderer {
    pub r#type: String,
    pub text: Runs,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Runs {
    pub runs: (TitleRun,),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleRun {
    pub text: String,
}
