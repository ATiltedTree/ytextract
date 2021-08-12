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

                match alerts.0.alert_renderer.text() {
                    "The playlist does not exist." | "This channel does not exist." => {
                        Err(crate::Error::Youtube(crate::error::Youtube::NotFound))
                    }
                    "This playlist type is unviewable." => {
                        Err(crate::Error::Youtube(crate::error::Youtube::Unviewable))
                    }
                    e => unimplemented!("Unknown Error text: '{}'", e),
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
    pub text: Text,
}

impl AlertRenderer {
    fn text(&self) -> &str {
        match &self.text {
            Text::SimpleText(simple_text) => simple_text.simple_text.as_str(),
            Text::Runs(runs) => runs.runs.0.text.as_str(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Text {
    SimpleText(SimpleText),
    Runs(Runs),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleText {
    pub simple_text: String,
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
