use std::time::Duration;

use serde::Deserialize;
use serde_with::serde_as;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub contents: Option<Contents>,
    pub microformat: Option<Microformat>,
    pub alerts: Option<(Alert,)>,
    pub sidebar: Option<Sidebar>,
}

impl Root {
    pub fn videos(&self) -> impl Iterator<Item = &PlaylistItem> {
        self.contents
            .as_ref()
            .expect("No content was found")
            .two_column_browse_results_renderer
            .tabs
            .0
            .tab_renderer
            .content
            .section_list_renderer
            .contents
            .0
            .item_section_renderer
            .contents
            .0
            .playlist_video_list_renderer
            .contents
            .iter()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contents {
    pub two_column_browse_results_renderer: TwoColumnBrowseResultsRenderer,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoColumnBrowseResultsRenderer {
    pub tabs: (Tab,),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub tab_renderer: TabRenderer,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabRenderer {
    pub content: Content,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub section_list_renderer: SectionListRenderer,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionListRenderer {
    pub contents: (Content2,),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content2 {
    pub item_section_renderer: ItemSectionRenderer,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSectionRenderer {
    pub contents: (Content3,),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content3 {
    pub playlist_video_list_renderer: PlaylistVideoListRenderer,
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistVideoListRenderer {
    pub contents: Vec<PlaylistItem>,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub playlist_id: crate::playlist::Id,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum PlaylistItem {
    PlaylistVideoRenderer(PlaylistVideoRenderer),
    ContinuationItemRenderer(ContinuationItemRenderer),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationItemRenderer {
    pub continuation_endpoint: ContinuationEndpoint,
}

impl ContinuationItemRenderer {
    pub fn get(&self) -> &str {
        &self.continuation_endpoint.continuation_command.token
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationEndpoint {
    pub continuation_command: ContinuationCommand,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationCommand {
    pub token: String,
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistVideoRenderer {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub video_id: crate::video::Id,

    pub thumbnail: Thumbnails,
    pub title: Runs<TitleRun>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_byline_text: Option<Runs<BylineRun>>,

    #[serde_as(as = "Option<serde_with::DurationSeconds<String>>")]
    #[serde(default)]
    pub length_seconds: Option<Duration>,

    #[serde(default)]
    pub is_playable: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Runs<T: Clone> {
    pub runs: (T,),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TitleRun {
    pub text: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BylineRun {
    pub text: String,
    pub navigation_endpoint: NavigationEndpoint,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint {
    pub browse_endpoint: BrowseEndpoint,
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BrowseEndpoint {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub browse_id: crate::channel::Id,
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Microformat {
    pub microformat_data_renderer: MicroformatDataRenderer,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicroformatDataRenderer {
    pub title: String,
    pub description: String,
    pub thumbnail: Thumbnails,
    pub unlisted: bool,
}
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnails {
    pub thumbnails: Vec<crate::Thumbnail>,
}
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub alert_renderer: AlertRenderer,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AlertRenderer {
    pub r#type: String,
    pub text: Runs<TitleRun>,
}

impl std::error::Error for AlertRenderer {}

impl std::fmt::Display for AlertRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text.runs.0.text)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Sidebar {
    pub playlist_sidebar_renderer: PlaylistSidebarRenderer,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistSidebarRenderer {
    pub items: Vec<PlaylistSidebarItem>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum PlaylistSidebarItem {
    PlaylistSidebarPrimaryInfoRenderer(PlaylistSidebarPrimaryInfoRenderer),
    PlaylistSidebarSecondaryInfoRenderer(PlaylistSidebarSecondaryInfoRenderer),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistSidebarPrimaryInfoRenderer {
    pub stats: (VideoStats, ViewsStats, DateStats),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VideoStats {
    pub runs: (TitleRun, TitleRun),
}

impl VideoStats {
    pub fn as_number(&self) -> u64 {
        self.runs
            .0
            .text
            .parse()
            .expect("VideoStats text was not a number")
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ViewsStats {
    pub simple_text: String,
}

impl ViewsStats {
    pub fn as_number(&self) -> u64 {
        let (number, _) = self
            .simple_text
            .split_once(' ')
            .expect("No space in ViewsStats text");
        number
            .parse()
            .expect("ViewsStats text did not contain a number")
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DateStats {}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistSidebarSecondaryInfoRenderer {
    pub video_owner: VideoOwner,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VideoOwner {
    pub video_owner_renderer: VideoOwnerRenderer,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VideoOwnerRenderer {
    pub title: Runs<BylineRun>,
}

impl VideoOwnerRenderer {
    pub fn name(&self) -> &str {
        &self.title.runs.0.text
    }

    pub fn id(&self) -> crate::channel::Id {
        self.title
            .runs
            .0
            .navigation_endpoint
            .browse_endpoint
            .browse_id
    }
}
