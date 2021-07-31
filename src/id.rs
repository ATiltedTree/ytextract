//! A YouTube Id

macro_rules! define_id {
    ($len:literal, $doc:literal, [$($prefix:literal),*,]) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        #[doc = $doc]
        pub struct Id([u8; $len]);

        impl std::ops::Deref for Id {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                std::str::from_utf8(&self.0[..]).expect("Id was invalid UTF-8")
            }
        }

        impl std::fmt::Debug for Id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::str::from_utf8(&self.0[..]).expect("Id was invalid UTF-8").fmt(f)
            }
        }

        impl std::fmt::Display for Id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::str::from_utf8(&self.0[..]).expect("Id was invalid UTF-8").fmt(f)
            }
        }

        impl<'de> serde::Deserialize<'de> for Id {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_str(IdVisitor)
            }
        }

        struct IdVisitor;

        impl<'de> serde::de::Visitor<'de> for IdVisitor {
            type Value = Id;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a id of length {}", $len)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(serde::de::Error::custom)
            }
        }

        impl serde::Serialize for Id {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                std::str::from_utf8(&self.0[..]).expect("Id was invalid UTF-8").serialize(serializer)
            }
        }

        impl std::str::FromStr for Id {
            type Err = crate::error::Id<$len>;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                use std::convert::TryInto;

                const PREFIXES: &[&str] = &[$($prefix),*];

                let value = PREFIXES
                    .iter()
                    .find_map(|prefix| value.strip_prefix(prefix))
                    // No Prefix matched. Possibly naked id. Length and
                    // correctness will be checked later.
                    .unwrap_or(value);

                value
                    .chars()
                    .all(crate::id::validate_char)
                    .then(|| value)
                    .ok_or_else(|| crate::error::Id::InvalidId(value.to_string()))
                    .and_then(|val| {
                        val.as_bytes()
                            .try_into()
                            .map_err(|_| crate::error::Id::InvalidLength(val.len()))
                    })
                    .map(Self)
            }
        }
    };
}

pub const fn validate_char(c: char) -> bool {
    matches!(c, '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' | '-')
}
