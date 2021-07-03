use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub on_response_received_actions: (OnResponseReceivedAction,),
}

impl Root {
    pub fn into_videos(self) -> impl Iterator<Item = super::playlist::PlaylistItem> {
        self.on_response_received_actions
            .0
            .append_continuation_items_action
            .continuation_items
            .into_iter()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnResponseReceivedAction {
    pub append_continuation_items_action: AppendContinuationItemsAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendContinuationItemsAction {
    pub continuation_items: Vec<super::playlist::PlaylistItem>,
}
