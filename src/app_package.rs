//! This module define a file format that can be read by a downstream tool that
//! bootloads a system.
//! It runs at a lower level of abstraction than the `script` module.
//! It includes all application data and metadata needed to bootload a system.
//! We serialize this data with serde to either json or CBOR.
//! As a file extension, we use `.gctapkg` or `.gct.apkgb`

use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    pub product_id: u16,
    pub node_id: u16,
    pub version: Version,

    #[serde(with = "base64")]
    pub image: Vec<u8>,
}

mod base64 {
    use base64::{prelude::BASE64_STANDARD, Engine};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let encoded = BASE64_STANDARD.encode(bytes);
            serializer.serialize_str(&encoded)
        } else {
            serializer.serialize_bytes(bytes)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Base64Vis;
        impl serde::de::Visitor<'_> for Base64Vis {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a base64 string")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                let decoded = BASE64_STANDARD.decode(v);
                decoded.map_err(E::custom)
            }
        }

        struct BytesVis;
        impl serde::de::Visitor<'_> for BytesVis {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a byte array")
            }

            fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
                Ok(v.to_vec())
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(Base64Vis)
        } else {
            deserializer.deserialize_bytes(BytesVis)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppPackage {
    app: Vec<App>,
}

impl AppPackage {
    pub fn new(apps: Vec<App>) -> Self {
        AppPackage { app: apps }
    }

    pub fn to_cbor(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        ciborium::into_writer(self, &mut ret).expect("This shouldn't fail");
        ret
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("This shouldn't fail")
    }
}

pub fn get_binary_file_extension() -> &'static str {
    "gctapkgb"
}

pub fn get_json_file_extension() -> &'static str {
    "gctapkg"
}
