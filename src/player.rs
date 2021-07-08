//! A YouTube Player associated with a video

use lazy_regex::regex_captures;
use regex::Regex;

macro_rules! cipher {
    ($name:ident) => {
        cipher!($name,)
    };

    ($name:ident, $($arg:ident: $index:expr),*) => {
        Box::new($name {
            $($arg: $index),*
        })
    };
}

/// A Error that can happen during [`Player`] parsing
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A Unknown cipher was encountered
    #[error("A Unknown Cipher was encountered: [{function_name}]: '{body}'")]
    UnknownCipher {
        /// The JS name of the cipher
        function_name: String,
        /// The body of the JS function
        body: String,
    },

    /// The cipher plan was unable to be found
    #[error("The cipher plan was unable to be found")]
    CipherPlanNotFound,

    /// A specific cipher function was not found
    #[error("Cipher function '{0}' was unable to be found")]
    CipherFunctionNotFound(String),

    /// A JS statement was unable to be parsed
    #[error("Unable to parse JS statement: '{0}'")]
    Statement(String),

    #[error("The player could not be found: '{0}'")]
    PlayerNotFound(reqwest::Error),
}

#[derive(Debug)]
pub struct Player {
    cipher_plan: CipherPlan,
}

impl Player {
    pub async fn from_url(http: &reqwest::Client, url: &str) -> Result<Self, Error> {
        let url = format!("https://youtube.com{}", url);
        log::trace!("Getting CipherPlan[{}]", url);
        let body = http
            .get(&url)
            .send()
            .await
            .map_err(Error::PlayerNotFound)?
            .error_for_status()
            .map_err(Error::PlayerNotFound)?
            .text()
            .await
            .map_err(Error::PlayerNotFound)?;

        log::trace!("Got CipherPlan[{}]", url);

        Ok(Self {
            cipher_plan: CipherPlan::from_body(&body)?,
        })
    }

    /// Get the [`CipherPlan`] of the player
    pub fn cipher(&self) -> &CipherPlan {
        &self.cipher_plan
    }
}

#[derive(Debug, Default)]
pub struct CipherPlan {
    ciphers: Vec<Box<dyn Cipher>>,
}

impl CipherPlan {
    /// Run the plan on a provided signature
    pub fn run(&self, signature: String) -> String {
        let mut signature = signature.into_bytes();
        log::trace!("Deciphering: '{:?}'", signature);

        for cipher in &self.ciphers {
            cipher.decipher(&mut signature);
        }

        log::trace!("Deciphered: '{:?}'", signature);

        String::from_utf8(signature).expect("Chipher result is invalid UTF-8")
    }

    fn from_body(body: &str) -> Result<Self, Error> {
        let (_, decipher_body) = regex_captures!(
            r#"\w+=function\(\w+\)\{\w+=\w+\.split\(""\);(.*)?return\s+\w+\.join\(""\)\};"#,
            body
        )
        .ok_or(Error::CipherPlanNotFound)?;

        let ciphers: Vec<Box<dyn Cipher>> = decipher_body
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| -> Result<Box<dyn Cipher>, Error> {
                let (_, function_name, arg) = regex_captures!(r"\w+\.(\w+)\(\w+,(\w+)\)", s)
                    .ok_or_else(|| Error::Statement(s.to_string()))?;

                let body_exp = Regex::new(&format!(
                    r"\b{}:function\([\w,]+\)\{{(.*?)\}}",
                    regex::escape(function_name)
                ))
                .expect("Function regex was not parsable");

                let body = &body_exp
                    .captures(body)
                    .ok_or_else(|| Error::CipherFunctionNotFound(function_name.to_string()))?[1];

                match body {
                    reverse if reverse.contains("reverse") => Ok(cipher! { ReverseCipher }),
                    splice if splice.contains("splice") => Ok(cipher! {
                        SpliceCipher,
                        index: arg.parse().expect("SpliceCipher argument was not an integer")
                    }),
                    swap if swap.contains('%') => Ok(cipher! {
                        SwapCipher,
                        index: arg.parse().expect("SwapCipher argument was not an integer")
                    }),
                    body => Err(Error::UnknownCipher {
                        function_name: function_name.to_string(),
                        body: body.to_string(),
                    }),
                }
            })
            .collect::<Result<_, Error>>()?;

        Ok(Self { ciphers })
    }
}

/// A JS Cipher implemented in Rust
trait Cipher: std::fmt::Debug + Sync + Send {
    /// Deciphers the input according to the JS function
    fn decipher(&self, input: &mut Vec<u8>);
}

#[derive(Debug)]
struct SpliceCipher {
    index: usize,
}

impl Cipher for SpliceCipher {
    fn decipher(&self, input: &mut Vec<u8>) {
        input.drain(..self.index);
    }
}

#[derive(Debug)]
struct SwapCipher {
    index: usize,
}

impl Cipher for SwapCipher {
    fn decipher(&self, input: &mut Vec<u8>) {
        input.swap(0, self.index);
    }
}

#[derive(Debug)]
struct ReverseCipher;

impl Cipher for ReverseCipher {
    fn decipher(&self, input: &mut Vec<u8>) {
        input.reverse();
    }
}
