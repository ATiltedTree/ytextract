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
define_test!(live_stream_recording, "rsAAeyAr-9Y");
define_test!(high_quality_streams, "V5Fsj_sCKdg");
define_test!(dash_manifest, "AI7ULzgf8RU");
define_test!(vr, "-xNN-bJQ4vI");
define_test!(hdr, "vX2vsvdq8nw");
define_test!(rating_disabled, "5VGm0dczmHc");
define_test!(subtitles, "YltHGKX80Y8");
define_test!(age_restricted, "SkRSXFQerZs");

mod embed_restricted {
    use super::CLIENT;

    define_test!(youtube, "_kmeFXjjGfk");
    define_test!(author, "MeJVWBSsPAY");
}

mod error {
    use super::CLIENT;

    macro_rules! define_test {
        ($fn:ident, $id:literal, $error:ident) => {
            #[async_std::test]
            async fn $fn() -> Result<(), Box<dyn std::error::Error>> {
                let id = $id.parse()?;
                assert!(matches!(
                    CLIENT.streams(id).await,
                    Err(ytextract::Error::Youtube(ytextract::error::Youtube::$error)),
                ));
                Ok(())
            }
        };
    }

    define_test!(not_found, "L_VmQZtLVID", NotFound);
    define_test!(private, "ZGdLIwrGHG8", Private);
    define_test!(required_purchase, "p3dDcKOFXQg", PurchaseRequired);
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

        match CLIENT.streams(id).await {
            Ok(_) => panic!("got OK"),
            Err(ytextract::Error::Youtube(ytextract::error::Youtube::CopyrightClaim {
                claiment,
            })) if claiment == "Richard DiBacco" => {}
            Err(other) => panic!("{:#?}", other),
        }
        Ok(())
    }
}
