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
    let channel = video.channel();
    assert_eq!(channel.id(), "UCXuqSBlHAE6Xw-yeJA0Tunw".parse()?);
    assert_eq!(channel.name(), "Linus Tech Tips");
    assert!(!video.description().is_empty());
    assert!(video.views() >= 1_068_917);

    let ratings = video.ratings();
    if let Ratings::Allowed { likes, dislikes } = ratings {
        assert!(likes >= 51_745);
        assert!(dislikes >= 622);
    } else {
        unreachable!();
    }

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

mod error {
    use super::CLIENT;

    macro_rules! define_test {
        ($fn:ident, $id:literal, $error:ident) => {
            #[async_std::test]
            async fn $fn() -> Result<(), Box<dyn std::error::Error>> {
                let id = $id.parse()?;
                assert!(matches!(
                    CLIENT.video(id).await,
                    Err(ytextract::Error::Youtube(ytextract::error::Youtube::$error)),
                ));
                Ok(())
            }
        };
    }

    define_test!(not_found, "L_VmQZtLVID", NotFound);
    define_test!(private, "ZGdLIwrGHG8", Private);
    define_test!(
        nudity_or_sexual,
        "-JVFs5w9V0U",
        NudityOrSexualContentViolation
    );
    define_test!(account_terminated, "Pfhpe6shO2U", AccountTerminated);
    define_test!(removed_by_uploader, "N3QlpdWUpHo", RemovedByUploader);
    define_test!(tos_violation, "tJievCeKBs0", TermsOfServiceViolation);

    #[async_std::test]
    async fn copyright_claim() -> Result<(), Box<dyn std::error::Error>> {
        let id = "6MNavkSGntQ".parse()?;

        match CLIENT.video(id).await {
            Ok(_) => panic!("got OK"),
            Err(ytextract::Error::Youtube(ytextract::error::Youtube::CopyrightClaim {
                claiment,
            })) if claiment == "Richard DiBacco" => {}
            Err(other) => panic!("{:#?}", other),
        }
        Ok(())
    }
}
