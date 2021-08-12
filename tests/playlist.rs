use std::sync::Arc;

use futures::StreamExt;
use once_cell::sync::Lazy;

static CLIENT: Lazy<Arc<ytextract::Client>> =
    Lazy::new(|| async_std::task::block_on(ytextract::Client::new()).unwrap());

#[async_std::test]
async fn get() -> Result<(), Box<dyn std::error::Error>> {
    let playlist = CLIENT
        .playlist("PLCSusC_jlo14BH5hHnOh9b0O18HtGT3eP".parse()?)
        .await?;

    assert_eq!(playlist.id(), "PLCSusC_jlo14BH5hHnOh9b0O18HtGT3eP".parse()?);
    assert_eq!(playlist.title(), "ytextract - test");
    assert_eq!(playlist.description(), "A");
    let channel = playlist.channel().expect("Channel missing");
    assert_eq!(channel.name(), "ATiltedTree");
    assert_eq!(channel.id(), "UCZqdX9k5eyv1aO7i2746bXg".parse()?);
    assert_eq!(channel.id(), channel.upgrade().await?.id());
    assert!(playlist.unlisted());
    assert!(!playlist.thumbnails().is_empty());
    #[allow(clippy::absurd_extreme_comparisons)]
    #[allow(unused_comparisons)]
    let views = playlist.views() >= 0;
    assert!(views);
    assert_eq!(playlist.length(), 121);

    let videos: Vec<_> = playlist.videos().collect().await;

    assert_eq!(videos.len(), 121);

    Ok(())
}

#[async_std::test]
async fn eq() -> Result<(), Box<dyn std::error::Error>> {
    let playlist1 = CLIENT
        .playlist("PLCSusC_jlo15M6x0Ot8gznM-QA8CriNk4".parse()?)
        .await?;
    let playlist2 = CLIENT
        .playlist("PLCSusC_jlo15M6x0Ot8gznM-QA8CriNk4".parse()?)
        .await?;

    assert_eq!(playlist1, playlist2);

    Ok(())
}

#[async_std::test]
async fn eq_channel() -> Result<(), Box<dyn std::error::Error>> {
    let playlist = CLIENT
        .playlist("PLCSusC_jlo15M6x0Ot8gznM-QA8CriNk4".parse()?)
        .await?;

    assert_eq!(playlist.channel(), playlist.channel());

    Ok(())
}

#[async_std::test]
async fn video() -> Result<(), Box<dyn std::error::Error>> {
    let playlist = CLIENT
        .playlist("PLCSusC_jlo14F22jss8ZtDLbpmRQIVLzr".parse()?)
        .await?;

    let videos: Vec<_> = playlist.videos().collect().await;

    for (i, video) in videos.into_iter().enumerate() {
        match i {
            0 => {
                let video = video?;
                assert_eq!(video.id(), "1_ozXudbN-4".parse()?);
                assert_eq!(video.title(), "Team Grimoire - C18H27NO3");
                assert_eq!(video.length(), std::time::Duration::from_secs(5 * 60 + 38));
                assert!(!video.thumbnails().is_empty());
                assert_eq!(video.channel().id(), "UCkc7SaDsN0MS6GNWFXEB0Lg".parse()?);
                assert_eq!(video.upgrade().await?.id(), video.id());
                assert!(video.streams().await?.next().is_some());
                assert_eq!(video.clone(), video);
            }
            _ => panic!("Too many videos"),
        }
    }

    Ok(())
}

mod metadata {
    use super::CLIENT;

    macro_rules! define_test {
        ($fn:ident, $id:literal, $($attr:meta)?) => {
            $(#[$attr])?
            #[async_std::test]
            async fn $fn() -> Result<(), Box<dyn std::error::Error>> {
                let id: ytextract::playlist::Id = $id.parse()?;
                let playlist = CLIENT.playlist(id.clone()).await?;
                assert_eq!(playlist.id(), id);
                Ok(())
            }
        };
        ($fn:ident, $id:literal) => {
            define_test!($fn, $id,);
        };
    }

    define_test!(normal, "PLI5YfMzCfRtZ8eV576YoY3vIYrHjyVm_e");
    define_test!(old, "PL601B2E69B03FAB9D");
    define_test!(no_videos, "PL4lCao7KL_QFodcLWhDpGCYnngnHtQ-Xf");
    define_test!(youtube_mix, "RDCLAK5uy_lf8okgl2ygD075nhnJVjlfhwp8NsUgEbs");
}

mod videos {
    use super::CLIENT;
    use futures::stream::StreamExt;

    macro_rules! define_test {
        ($fn:ident, $id:literal, $($attr:meta)?) => {
            $(#[$attr])?
            #[async_std::test]
            async fn $fn() -> Result<(), Box<dyn std::error::Error>> {
                let id = $id.parse()?;
                let playlist = CLIENT.playlist(id).await?;
                let videos = playlist.videos();
                futures::pin_mut!(videos);
                assert!(videos.next().await.is_some());
                Ok(())
            }
        };
        ($fn:ident, $id:literal) => {
            define_test!($fn, $id,);
        };
    }

    define_test!(normal, "PLI5YfMzCfRtZ8eV576YoY3vIYrHjyVm_e");
    define_test!(very_long, "PLWwAypAcFRgKFlxtLbn_u14zddtDJj3mk");
    define_test!(audio_book, "OLAK5uy_mtOdjCW76nDvf5yOzgcAVMYpJ5gcW5uKU");
    define_test!(channel_uploads, "UUTMt7iMWa7jy0fNXIktwyLA");
    define_test!(song_album, "OLAK5uy_lLeonUugocG5J0EUAEDmbskX4emejKwcM");
    define_test!(old, "PL601B2E69B03FAB9D");
    define_test!(youtube_mix, "RDCLAK5uy_lf8okgl2ygD075nhnJVjlfhwp8NsUgEbs");
}

mod error {
    use futures::StreamExt;

    use super::CLIENT;

    macro_rules! define_test {
        ($fn:ident, $id:literal, $error:ident) => {
            #[async_std::test]
            async fn $fn() -> Result<(), Box<dyn std::error::Error>> {
                let id = $id.parse()?;
                let playlist = CLIENT.playlist(id).await;
                match playlist {
                    Err(ytextract::Error::Youtube(ytextract::error::Youtube::$error)) => {}
                    err => panic!("{:#?}", err),
                }
                Ok(())
            }
        };
    }

    define_test!(not_found, "PLI5YfMzCfRtZ8eV576YoY3vIYrHjyVm_F", NotFound);
    define_test!(unviewable, "RDGMEMQ1dJ7wXfLlqCjwV0xfSNbA", Unviewable);
    define_test!(private, "PLCSusC_jlo16bxXlQLScDy4kgdLhhQP8A", NotFound);
    define_test!(deleted, "PLCSusC_jlo16qHzHLY6jmCWG_ov7R2hMv", NotFound);

    #[async_std::test]
    async fn video() -> Result<(), Box<dyn std::error::Error>> {
        let playlist = CLIENT
            .playlist("PLCSusC_jlo146Bv2QRvW7jvV0wZYiSf8N".parse()?)
            .await?;
        use ytextract::playlist::video::UnavailabilityReason;

        const ERROR_IDS: [(UnavailabilityReason, &str); 2] = [
            (UnavailabilityReason::Private, "Tk4J8s5T790"),
            (UnavailabilityReason::Deleted, "MwCXB2byk58"),
        ];

        let videos: Vec<_> = playlist.videos().collect().await;

        for (video, (reason, id)) in videos.iter().zip(ERROR_IDS) {
            assert_eq!(
                video,
                &Err(ytextract::playlist::video::Error {
                    reason,
                    id: id.parse()?
                })
            );
        }

        Ok(())
    }
}
