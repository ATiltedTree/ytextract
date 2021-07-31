use std::sync::Arc;

use once_cell::sync::Lazy;

static CLIENT: Lazy<Arc<ytextract::Client>> =
    Lazy::new(|| async_std::task::block_on(ytextract::Client::new()).unwrap());

#[async_std::test]
async fn get() -> Result<(), Box<dyn std::error::Error>> {
    let id = "UCdktGrgQlqxPsvHo6cHF0Ng".parse()?;
    let channel = CLIENT.channel(id).await?;

    assert_eq!(channel.id(), id);
    assert_eq!(channel.name(), "Cooksie");
    assert!(!channel.description().is_empty());
    assert_eq!(channel.country(), Some("United States"));
    assert!(channel.avatar().next().is_some());
    assert!(channel.banner().next().is_some());
    assert!(channel.views() >= 90_969_900);
    assert!(channel.subscribers() >= 146_000);
    assert!(channel.uploads().await.is_ok());
    assert!(channel.badges().next().is_some());

    Ok(())
}

mod metadata {
    use super::CLIENT;

    macro_rules! define_test {
        ($fn:ident, $id:literal, $($attr:meta)?) => {
            $(#[$attr])?
            #[async_std::test]
            async fn $fn() -> Result<(), Box<dyn std::error::Error>> {
                let id: ytextract::channel::Id = $id.parse()?;
                let channel = CLIENT.channel(id.clone()).await?;
                assert_eq!(channel.id(), id);
                Ok(())
            }
        };
        ($fn:ident, $id:literal) => {
            define_test!($fn, $id,);
        };
    }

    define_test!(video_game_ost, "UC46807r_RiRjH8IU-h_DrDQ");
    define_test!(monstercat_uncaged, "UCJ6td3C9QlPO9O_J5dF4ZzA");
    define_test!(billie, "UCiGm_E4ZwYSHV3bcW1pnSeQ");
    define_test!(no_description, "UCydibA_hF_LGiMYNgDrj50A");
}

mod uploads {
    use super::CLIENT;

    macro_rules! define_test {
        ($fn:ident, $id:literal, $($attr:meta)?) => {
            $(#[$attr])?
            #[async_std::test]
            async fn $fn() -> Result<(), Box<dyn std::error::Error>> {
                let id: ytextract::channel::Id = $id.parse()?;
                let playlist = CLIENT.playlist(id.uploads()).await?;
                assert_eq!(playlist.channel().expect("channel").id(), id);
                Ok(())
            }
        };
        ($fn:ident, $id:literal) => {
            define_test!($fn, $id,);
        };
    }

    define_test!(video_game_ost, "UC46807r_RiRjH8IU-h_DrDQ");
    define_test!(monstercat_uncaged, "UCJ6td3C9QlPO9O_J5dF4ZzA");
    define_test!(billie, "UCiGm_E4ZwYSHV3bcW1pnSeQ");
}

mod badges {
    use super::CLIENT;

    macro_rules! define_test {
        ($fn:ident, $id:literal, $badge:ident, $($attr:meta)?) => {
            $(#[$attr])?
            #[async_std::test]
            async fn $fn() -> Result<(), Box<dyn std::error::Error>> {
                let channel = CLIENT.channel($id.parse()?).await?;
                assert_eq!(channel.badges().next(), Some(ytextract::channel::Badge::$badge));
                Ok(())
            }
        };
        ($fn:ident, $id:literal, $badge:ident) => {
            define_test!($fn, $id, $badge,);
        };
    }

    define_test!(artist, "UCiGm_E4ZwYSHV3bcW1pnSeQ", VerifiedArtist);
    define_test!(verified, "UCDxS8VeAQhnHJc6B5jE3KHg", Verified);
}

mod error {
    use super::CLIENT;

    macro_rules! define_test {
        ($fn:ident, $id:literal, $error:ident) => {
            #[async_std::test]
            async fn $fn() -> Result<(), Box<dyn std::error::Error>> {
                let id = $id.parse()?;
                let channel = CLIENT.channel(id).await;
                assert!(matches!(
                    channel,
                    Err(ytextract::Error::Youtube(ytextract::error::Youtube::$error)),
                ));
                Ok(())
            }
        };
    }

    define_test!(not_found, "UC5CwaMl1eIgY8h02uZw7u8F", NotFound);
}
