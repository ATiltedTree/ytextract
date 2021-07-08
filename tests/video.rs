use std::sync::Arc;

use once_cell::sync::Lazy;
use ytextract::video::Ratings;

static CLIENT: Lazy<Arc<ytextract::Client>> =
    Lazy::new(|| async_std::task::block_on(ytextract::Client::new()).unwrap());

#[async_std::test]
async fn get() -> Result<(), Box<dyn std::error::Error>> {
    let video = CLIENT
        .video("https://www.youtube.com/watch?v=7B2PIVSWtJA".parse()?)
        .await?;

    assert_eq!(
        video.title(),
        "I Sent Corridor Digital the WORST VFX Workstation"
    );

    assert_eq!(video.id(), "7B2PIVSWtJA".parse()?);
    assert_eq!(video.duration(), std::time::Duration::from_secs(1358));
    assert_eq!(
        video.keywords(),
        &vec![
            "photoshop",
            "adobe",
            "1.0",
            "macintosh",
            "apple",
            "lc",
            "475",
            "quadra",
            "performa",
            "classic",
            "system 7.5",
            "macos",
            "ossc",
            "vga",
            "vfx",
            "editing",
            "challenge",
            "corridor digital",
            "collab",
            "ftp",
            "fetch",
            "icab",
            "marathon",
            "oregon trail",
            "nightmare fuel",
            "scsi2sd"
        ]
    );
    assert_eq!(video.channel_id(), "UCXuqSBlHAE6Xw-yeJA0Tunw".parse()?);
    assert_eq!(video.author(), "Linus Tech Tips");
    assert!(!video.description().is_empty());
    assert!(video.views() >= 1_068_917);

    let ratings = video.ratings();
    if let Ratings::Allowed { likes, dislikes } = ratings {
        assert!(likes >= 51_745);
        assert!(dislikes >= 622);
    } else {
        unreachable!();
    }

    assert!(!video.private());
    assert!(!video.live());
    assert!(!video.thumbnails().is_empty());
    assert!(!video.age_restricted());
    assert!(!video.unlisted());
    assert!(video.family_safe());
    assert_eq!(video.category(), "Science & Technology");
    assert_eq!(
        video.publish_date(),
        chrono::NaiveDate::from_ymd(2021, 4, 14)
    );
    assert_eq!(
        video.upload_date(),
        chrono::NaiveDate::from_ymd(2021, 4, 14)
    );

    Ok(())
}

macro_rules! define_test {
    ($fn:ident, $id:literal) => {
        #[async_std::test]
        async fn $fn() -> Result<(), Box<dyn std::error::Error>> {
            let id = $id.parse()?;
            let video = CLIENT.video(id).await?;
            assert_eq!(video.id(), id);
            Ok(())
        }
    };
}

define_test!(normal, "9bZkp7q19f0");
define_test!(live_stream, "5qap5aO4i9A");
define_test!(live_stream_recording, "rsAAeyAr-9Y");
define_test!(high_quality_streams, "V5Fsj_sCKdg");
define_test!(dash_manifest, "AI7ULzgf8RU");
define_test!(vr, "-xNN-bJQ4vI");
define_test!(hdr, "vX2vsvdq8nw");
define_test!(age_restricted, "SkRSXFQerZs");
define_test!(rating_disabled, "5VGm0dczmHc");
define_test!(required_purchase, "p3dDcKOFXQg");
define_test!(subtitles, "YltHGKX80Y8");

mod embed_restricted {
    use super::CLIENT;

    define_test!(youtube, "_kmeFXjjGfk");
    define_test!(author, "MeJVWBSsPAY");
    define_test!(age_restricted, "hySoCSoH-g8");
}