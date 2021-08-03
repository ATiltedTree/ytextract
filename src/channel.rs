//! Channel types.

use std::sync::Arc;

use crate::{
    youtube::{
        browse,
        innertube::{Browse, ChannelPage},
    },
    Client,
};

define_id! {
    24,
    "An Id describing a [`Channel`]",
    [
        "https://www.youtube.com/channel/",
    ]
}

impl Id {
    /// Get the playlist id that contains the uploads for this channel
    pub fn uploads(self) -> crate::playlist::Id {
        String::from(&*self)
            .chars()
            .enumerate()
            // Turn `UCdktGrgQlqxPsvHo6cHF0Ng`
            // into `UUdktGrgQlqxPsvHo6cHF0Ng`
            .map(|(i, ch)| if i == 1 { 'U' } else { ch })
            .collect::<String>()
            .parse()
            .expect("Unable to convert")
    }
}

/// A badge that a [`Channel`] can have
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Badge {
    /// A artist that is verified
    VerifiedArtist,

    /// A channel that is verified
    Verified,
}

impl Badge {
    pub(crate) fn from(badge: &browse::channel::Badge) -> Self {
        match badge.metadata_badge_renderer.style.as_str() {
            "BADGE_STYLE_TYPE_VERIFIED_ARTIST" => Self::VerifiedArtist,
            "BADGE_STYLE_TYPE_VERIFIED" => Self::Verified,
            badge => unreachable!("Unknown badge: '{}'", badge),
        }
    }
}

/// A Channel
#[derive(Clone)]
pub struct Channel {
    client: Arc<Client>,
    response: browse::channel::about::Root,
}

impl Channel {
    pub(crate) async fn get(client: Arc<Client>, id: Id) -> crate::Result<Self> {
        let response: browse::channel::about::Result = client
            .api
            .browse(Browse::Channel {
                id,
                page: ChannelPage::About,
            })
            .await?;

        let response = response.into_std()?;

        Ok(Self {
            client: Arc::clone(&client),
            response,
        })
    }

    fn contents(&self) -> &browse::channel::about::ChannelAboutFullMetadataRenderer {
        &self
            .response
            .contents()
            .section_list_renderer
            .contents
            .0
            .item_section_renderer
            .contents
            .0
            .channel_about_full_metadata_renderer
    }

    fn header(&self) -> &browse::channel::C4TabbedHeaderRenderer {
        &self.response.header.c4_tabbed_header_renderer
    }

    /// The [`Id`] of a channel
    pub fn id(&self) -> Id {
        self.header().channel_id
    }

    /// The name of the channel
    pub fn name(&self) -> &str {
        &self.header().title
    }

    /// The description of the channel
    pub fn description(&self) -> &str {
        &self.contents().description.simple_text
    }

    /// The country that this channel resides in
    pub fn country(&self) -> Option<&str> {
        self.contents()
            .country
            .as_ref()
            .map(|x| x.simple_text.as_str())
    }

    /// The views that this channel received
    pub fn views(&self) -> u64 {
        self.contents()
            .view_count_text
            .simple_text
            .split_once(' ')
            .expect("No space in view count")
            .0
            .replace(',', "")
            .parse()
            .expect("not int")
    }

    /// The amount of subscribers this channel has.
    pub fn subscribers(&self) -> u64 {
        parse_amount(
            self.header()
                .subscriber_count_text
                .simple_text
                .split_once(' ')
                .expect("no space in subscriber_count_text")
                .0,
        )
        .expect("Unable to parse subscriber count")
    }

    /// The avatar of the channel in various sizes
    pub fn avatar(&self) -> impl Iterator<Item = &crate::Thumbnail> {
        self.header().avatar.thumbnails.iter()
    }

    /// The banner of the channel in various sizes
    pub fn banner(&self) -> impl Iterator<Item = &crate::Thumbnail> {
        self.header().banner.thumbnails.iter()
    }

    /// The [`Badges`](Badge) of this channel.
    pub fn badges(&self) -> impl Iterator<Item = Badge> + '_ {
        self.header().badges.iter().map(Badge::from)
    }

    /// The uploads of a channel in a [`Playlist`](crate::Playlist)
    pub async fn uploads(&self) -> crate::Result<crate::Playlist> {
        self.client.playlist(self.id().uploads()).await
    }

    // TODO: Playlist
    // TODO: Channels
}

impl std::fmt::Debug for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Channel")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("description", &self.description())
            .field("country", &self.country())
            .field("views", &self.views())
            .field("subscribers", &self.subscribers())
            .field("avatar", &self.avatar().collect::<Vec<_>>())
            .field("banner", &self.banner().collect::<Vec<_>>())
            .field("badges", &self.badges().collect::<Vec<_>>())
            .finish()
    }
}

impl PartialEq for Channel {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for Channel {}

fn parse_amount(value: &str) -> Option<u64> {
    let last = value.chars().last()?;
    if last.is_numeric() {
        value.parse().ok()
    } else {
        let val = &value[..value.len() - 1];
        let val: f64 = val.parse().ok()?;
        let mul = match last {
            'K' => 1_000.0,
            'M' => 1_000_000.0,
            modifier => unreachable!("Unknown modifier '{}'", modifier),
        };

        Some((val * mul) as u64)
    }
}
