use std::sync::Arc;

use once_cell::sync::Lazy;

static CLIENT: Lazy<Arc<ytextract::Client>> =
    Lazy::new(|| async_std::task::block_on(ytextract::Client::new()).unwrap());

macro_rules! define_test {
    ($fn:ident, $id:literal, $($attr:meta)?) => {
        $(#[$attr])?
        #[async_std::test]
        async fn $fn() -> Result<(), Box<dyn std::error::Error>> {
            let id = $id.parse()?;
            let mut streams = CLIENT.streams(id).await?;
            assert!(streams.next().is_some());
            Ok(())
        }
    };
    ($fn:ident, $id:literal) => {
        define_test!($fn, $id,);
    }
}

define_test!(normal, "9bZkp7q19f0");
define_test!(unlisted, "ZGdLIwrGHG8");
define_test!(live_stream_recording, "rsAAeyAr-9Y");
define_test!(high_quality_streams, "V5Fsj_sCKdg");
define_test!(dash_manifest, "AI7ULzgf8RU");
define_test!(vr, "-xNN-bJQ4vI");
define_test!(hdr, "vX2vsvdq8nw");
define_test!(age_restricted, "SkRSXFQerZs");
define_test!(rating_disabled, "5VGm0dczmHc");
define_test!(subtitles, "YltHGKX80Y8");

mod embed_restricted {
    use super::CLIENT;

    define_test!(youtube, "_kmeFXjjGfk");
    define_test!(author, "MeJVWBSsPAY");
    define_test!(age_restricted, "hySoCSoH-g8");
}
