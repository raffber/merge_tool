//! This module define a file format that can be read by a downstream tool that
//! bootloads a system.
//! It runs at a lower level of abstraction than the `script` module.
//! It includes all application data and metadata needed to bootload a system.
//! We serialize this data with serde to either json or CBOR.
//! As a file extension, we use `.gctapkg` or `.gct.apkgb`

use std::path::Path;

use semver::Version;
use serde::{Deserialize, Serialize};

pub const BINARY_FILE_EXTENSION: &'static str = "gctapkg";

pub const JSON_FILE_EXTENSION: &'static str = "gctapkg.json";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct App {
    pub product_id: u16,
    pub node_id: u8,
    pub version: Version,
    pub crc: u32,

    pub image: Vec<Section>,
}

impl App {
    pub fn from_loaded_firmware(
        product_id: u16,
        loaded_fw: &crate::process::LoadedFirmware,
    ) -> Self {
        let app = &loaded_fw.app;
        let config = &loaded_fw.config;
        let image = Section::new(app.range.begin, app.data.clone());

        App {
            product_id: product_id,
            node_id: config.node_id,
            version: config.version.clone().unwrap_or(Version::new(0, 0, 0)),
            crc: app.read_u32(0),
            image: vec![image],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Section {
    offset: u64,
    #[serde(with = "base64")]
    data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppPackage {
    pub app: Vec<App>,
}

impl AppPackage {
    pub fn new(apps: Vec<App>) -> Self {
        AppPackage { app: apps }
    }

    pub fn from_loaded_firmware_images(
        product_id: u16,
        fws: &crate::process::LoadedFirmwareImages,
    ) -> Self {
        let apps = fws
            .images
            .iter()
            .map(|fw| App::from_loaded_firmware(product_id, fw))
            .collect();
        AppPackage::new(apps)
    }

    pub fn to_cbor(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        ciborium::into_writer(self, &mut ret).expect("This shouldn't fail");
        ret
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("This shouldn't fail")
    }

    pub fn from_cbor(data: &[u8]) -> crate::Result<Self> {
        ciborium::from_reader(data).map_err(crate::Error::other)
    }

    pub fn from_json(data: &str) -> crate::Result<Self> {
        serde_json::from_str(data).map_err(crate::Error::other)
    }

    pub fn load_from_file(fpath: &Path) -> crate::Result<Self> {
        let ext = fpath.extension().unwrap_or_default();
        let data = std::fs::read(fpath)?;
        match ext.to_str() {
            Some(JSON_FILE_EXTENSION) => Self::from_json(&String::from_utf8_lossy(&data)),
            _ => Self::from_cbor(&data),
        }
    }
}

impl Section {
    pub fn new(offset: u64, data: Vec<u8>) -> Self {
        Section { offset, data }
    }
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

#[cfg(test)]
mod test {
    use super::*;
    use semver::Version;

    #[test]
    fn test_app_package() {
        let app = App {
            product_id: 0x1234,
            node_id: 0x12,
            version: Version::new(1, 2, 3),
            crc: 0x12345678,
            image: vec![Section::new(0, vec![0x12, 0x34, 0x56, 0x78])],
        };

        let app_package = AppPackage::new(vec![app]);

        let cbor = app_package.to_cbor();
        let json = app_package.to_json();

        let app_package_cbor: AppPackage =
            ciborium::from_reader(&cbor[..]).expect("This shouldn't fail");
        let app_package_json: AppPackage =
            serde_json::from_str(&json).expect("This shouldn't fail");

        assert_eq!(app_package, app_package_cbor);
        assert_eq!(app_package, app_package_json);
    }
}
