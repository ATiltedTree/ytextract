use tokio_stream::StreamExt;

#[tokio::test]
async fn get() -> Result<(), Box<dyn std::error::Error>> {
    let client = crate::Client::new().await?;

    let playlist = client
        .playlist(
            "https://www.youtube.com/playlist?list=PLCSusC_jlo15M6x0Ot8gznM-QA8CriNk4".parse()?,
        )
        .await?;

    assert_eq!(playlist.title(), "V1");
    assert_eq!(playlist.description(), "A");
    assert!(!playlist.thumbnails().is_empty());
    assert!(playlist.unlisted());

    let videos: Vec<_> = playlist.videos().collect().await;

    assert!(videos.len() >= 84);

    Ok(())
}

#[tokio::test]
async fn long() -> Result<(), Box<dyn std::error::Error>> {
    let client = crate::Client::new().await?;

    let playlist = client
        .playlist(
            "https://www.youtube.com/playlist?list=PLCSusC_jlo14_7xep5gastSaLmqDeS2cZ".parse()?,
        )
        .await?;

    assert_eq!(playlist.title(), "uhhhhhh");
    assert_eq!(playlist.description(), "");
    assert!(!playlist.thumbnails().is_empty());
    assert!(playlist.unlisted());

    let videos: Vec<_> = playlist.videos().collect().await;

    assert!(videos.len() >= 500);

    Ok(())
}
