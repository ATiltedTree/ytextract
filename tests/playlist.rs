use std::sync::Arc;

use once_cell::sync::Lazy;

static CLIENT: Lazy<Arc<ytextract::Client>> =
    Lazy::new(|| async_std::task::block_on(ytextract::Client::new()).unwrap());

#[async_std::test]
async fn get() -> Result<(), Box<dyn std::error::Error>> {
    let playlist = CLIENT
        .playlist("PLCSusC_jlo15M6x0Ot8gznM-QA8CriNk4".parse()?)
        .await?;

    assert_eq!(playlist.id(), "PLCSusC_jlo15M6x0Ot8gznM-QA8CriNk4".parse()?);
    assert_eq!(playlist.title(), "V1");
    assert_eq!(playlist.description(), "A");
    let channel = playlist.channel().expect("Channel missing");
    assert_eq!(channel.name(), "ATiltedTree");
    assert_eq!(channel.id(), "UCZqdX9k5eyv1aO7i2746bXg".parse()?);
    assert!(playlist.unlisted());
    assert!(!playlist.thumbnails().is_empty());
    assert!(playlist.views() >= 92);
    assert!(playlist.length() >= 85);

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
    use super::CLIENT;

    macro_rules! define_test {
        ($fn:ident, $id:literal, $error:ident) => {
            #[async_std::test]
            async fn $fn() -> Result<(), Box<dyn std::error::Error>> {
                let id = $id.parse()?;
                let playlist = CLIENT.playlist(id).await;
                assert!(matches!(
                    playlist,
                    Err(ytextract::Error::Youtube(ytextract::error::Youtube::$error)),
                ));
                Ok(())
            }
        };
    }

    define_test!(not_found, "PLI5YfMzCfRtZ8eV576YoY3vIYrHjyVm_F", NotFound);
    define_test!(unviewable, "RDGMEMQ1dJ7wXfLlqCjwV0xfSNbA", Unviewable);
}
