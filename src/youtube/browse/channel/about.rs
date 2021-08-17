use serde::Deserialize;

use crate::youtube::SimpleText;

pub type Result = super::Result<Content>;

pub type Root = super::Ok<Content>;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub section_list_renderer: ListRenderer<ItemSectionRenderer>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListRenderer<T> {
    pub contents: (T,),
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemSectionRenderer {
    pub item_section_renderer: ListRenderer<ChannelAbout>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChannelAbout {
    pub channel_about_full_metadata_renderer: ChannelAboutFullMetadataRenderer,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChannelAboutFullMetadataRenderer {
    #[serde(default)]
    pub description: SimpleText,
    pub view_count_text: Option<SimpleText>,
    pub country: Option<SimpleText>,
    pub joined_date_text: JoinedDateText,
}

impl ChannelAboutFullMetadataRenderer {
    pub fn views(&self) -> u64 {
        self.view_count_text
            .as_ref()
            .and_then(|x| {
                x.simple_text
                    .split_once(' ')
                    .map(|(start, _)| start.replace(',', "").parse().expect("not int"))
            })
            .unwrap_or(0)
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JoinedDateText {
    pub runs: (Text, Text),
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Text {
    pub text: String,
}
