use serde::Deserialize;

use crate::youtube::parse_subscribers;

pub mod about;

pub type Result<T> = super::Result<Ok<T>>;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Ok<T> {
    pub header: Header,
    pub metadata: Metadata,
    pub contents: Contents<T>,
}

impl<T> Ok<T> {
    pub fn contents(&self) -> &T {
        self.contents
            .two_column_browse_results_renderer
            .tabs
            .iter()
            .find_map(|x| match x {
                Tab::Some { tab_renderer } => Some(&tab_renderer.content),
                Tab::None {} => None,
            })
            .expect("channel response did not contain a opened tab")
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub c4_tabbed_header_renderer: C4TabbedHeaderRenderer,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct C4TabbedHeaderRenderer {
    pub title: String,
    pub channel_id: crate::channel::Id,
    pub avatar: Thumbnails,
    #[serde(default)]
    pub banner: Thumbnails,
    #[serde(default)]
    pub badges: Vec<Badge>,
    pub subscriber_count_text: Option<SimpleText>,
}

impl C4TabbedHeaderRenderer {
    pub fn subscribers(&self) -> Option<u64> {
        self.subscriber_count_text.as_ref().map(|x| {
            parse_subscribers(
                x.simple_text
                    .split_once(' ')
                    .expect("no space in subscriber_count_text")
                    .0,
            )
            .expect("Unable to parse subscriber count")
        })
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnails {
    pub thumbnails: Vec<crate::Thumbnail>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub channel_metadata_renderer: ChannelMetadataRenderer,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMetadataRenderer {
    pub is_family_safe: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Contents<T> {
    pub two_column_browse_results_renderer: TwoColumnBrowseResultsRenderer<T>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TwoColumnBrowseResultsRenderer<T> {
    pub tabs: Vec<Tab<T>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimpleText {
    pub simple_text: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Tab<T> {
    Some {
        #[serde(rename = "tabRenderer")]
        tab_renderer: TabRenderer<T>,
    },
    None {},
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TabRenderer<T> {
    pub content: T,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub metadata_badge_renderer: MetadataBadgeRenderer,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MetadataBadgeRenderer {
    pub style: String,
}
