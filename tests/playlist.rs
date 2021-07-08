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
    assert_eq!(playlist.author(), Some("ATiltedTree"));
    assert_eq!(
        playlist.channel_id(),
        Some("UCZqdX9k5eyv1aO7i2746bXg".parse()?)
    );
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
    define_test!(mix, "RD1hu8-y6fKg0", ignore);
    define_test!(my_mix, "RDMMU-ty-2B02VY", ignore);
    define_test!(youtube_mix, "RDCLAK5uy_lf8okgl2ygD075nhnJVjlfhwp8NsUgEbs");
    define_test!(old, "PL601B2E69B03FAB9D");
}

mod videos {
    use super::CLIENT;
    use futures::stream::StreamExt;

    macro_rules! define_test {
        ($fn:ident, $id:literal) => {
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
    }

    define_test!(normal, "PLI5YfMzCfRtZ8eV576YoY3vIYrHjyVm_e");
    define_test!(very_long, "PLWwAypAcFRgKFlxtLbn_u14zddtDJj3mk");
    define_test!(audio_book, "OLAK5uy_mtOdjCW76nDvf5yOzgcAVMYpJ5gcW5uKU");
    define_test!(youtube_mix, "RDCLAK5uy_lf8okgl2ygD075nhnJVjlfhwp8NsUgEbs");
    define_test!(channel_uploads, "UUTMt7iMWa7jy0fNXIktwyLA");
    define_test!(song_album, "OLAK5uy_lLeonUugocG5J0EUAEDmbskX4emejKwcM");
    define_test!(old, "PL601B2E69B03FAB9D");
}
