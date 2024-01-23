use std::fs::{create_dir_all, File};
use std::io::Write;
use std::iter::repeat;
use std::path::{Path, PathBuf};

use byteorder::{ByteOrder, LittleEndian};
use chrono::{DateTime, Utc};
use merge_tool::config::{AddressRange, Config};
use merge_tool::crc::crc32;
use merge_tool::intel_hex;
use merge_tool::process::{self, save_info, save_merged_firmware_images};

fn save_hex(path: &str, data: &[u8], range: &AddressRange) {
    let serialized = intel_hex::serialize(false, range, data);
    File::create(path)
        .unwrap()
        .write_all(serialized.as_bytes())
        .unwrap();
}

fn create_test_data() {
    let mut data: Vec<_> = (1u8..0x50).collect();

    save_hex(
        "tests/btl_f1.hex",
        &data.clone(),
        &AddressRange::new(0xAA00, 0xAAFF),
    );
    save_hex(
        "tests/btl_f2.hex",
        &data.clone(),
        &AddressRange::new(0xAA00, 0xAAFF),
    );

    data[4 + 2] = 1; // firmware id
    data[4 + 4] = 3; // major
    data[4 + 5] = 0; // major msb
    data[4 + 6] = 5; // minor
    data[4 + 7] = 0; // minor msb
    data[4 + 8] = 4; // patch
    data[4 + 9] = 0; // patch
    data[4 + 10] = 0; // patch
    data[4 + 11] = 0; // patch
    save_hex(
        "tests/app_f1.hex",
        &data.clone(),
        &AddressRange::new(0xAB00, 0xABFF),
    );

    data[4 + 2] = 2; // firmware id
    data[4 + 4] = 3; // major
    data[4 + 6] = 8; // minor
    data[4 + 8] = 7; // patch
    save_hex(
        "tests/app_f2.hex",
        &data.clone(),
        &AddressRange::new(0xAB00, 0xABFF),
    );
}

struct IntegrationTest {
    config: Config,
    output_dir: PathBuf,
    config_dir: PathBuf,
}

impl IntegrationTest {
    fn new() -> IntegrationTest {
        create_test_data();
        let config_path = Path::new("tests/test.gctmrg");
        let config_path = std::fs::canonicalize(&config_path).unwrap();
        let config_dir = Config::get_config_dir(&config_path).unwrap();
        let mut config = Config::load_from_file(&config_path).unwrap();
        config.build_time = DateTime::<Utc>::from_timestamp(1000, 0).unwrap();
        let output_dir = config_dir.join("out");
        IntegrationTest {
            config,
            output_dir,
            config_dir,
        }
    }
}

#[test]
fn merge() {
    let test = IntegrationTest::new();
    let config = test.config;
    let config_dir = test.config_dir;
    let loaded = process::load_firmware_images(&config, &config_dir).unwrap();
    let fws = process::merge_all(&loaded).unwrap();

    // examine F1 firmware
    let fw = &fws.images[0].0;
    let fw_cfg = &loaded.images[0].config;
    assert_eq!(fw.data[256 + 6], 1);
    assert_eq!(fw.config.page_size, 64);
    assert_eq!(fw.range.begin, 0xAA00);
    assert_eq!(fw.range.end, 0xAC00);
    assert_eq!(fw.image_length(), 256 + 128); // full btl and first 128bytes of app (2 pages)
    assert_eq!(loaded.config.product_id, 0x605);
    assert_eq!(fw_cfg.node_id, 1);

    let version = fw_cfg.version.as_ref().unwrap();
    assert_eq!(version.major, 3);
    assert_eq!(version.minor, 5);
    assert_eq!(version.patch, 4);

    // reconstruct app data
    let mut data: Vec<_> = (1u8..0x50).collect();
    data.extend(repeat(0xFF).take(128 - 0x50 + 1));
    assert_eq!(data.len(), 128);
    data[4 + 2] = 1; // firmware id
    data[4 + 4] = 3; // major
    data[4 + 5] = 0;
    data[4 + 6] = 5; // minor
    data[4 + 7] = 0;
    data[4 + 8] = 4; // patch
    data[4 + 9] = 0;
    data[4 + 10] = 0;
    data[4 + 11] = 0;
    data[4 + 12] = 128; // length
    data[4 + 13] = 0;
    data[4 + 14] = 0;
    data[4 + 15] = 0;

    LittleEndian::write_u32(&mut data[4 + 18..4 + 22], 1000);
    LittleEndian::write_u16(&mut data[4 + 22..4 + 24], 0);

    // compute and compare CRC
    let ref_crc = crc32(&data[4..128]);
    let crc = fw.read_u32(256);
    assert_eq!(ref_crc, crc);

    let output_dir = config_dir.join("out");
    create_dir_all(&output_dir).unwrap();
    save_merged_firmware_images(&fws, &output_dir).unwrap()
}

#[test]
fn script() {
    let test = IntegrationTest::new();
    let loaded = process::load_firmware_images(&test.config, &test.config_dir).unwrap();
    let script = process::create_script(&loaded).unwrap();
    process::save_script(&script, &loaded, &test.output_dir).unwrap();
}

#[test]
fn info() {
    let test = IntegrationTest::new();
    let loaded = process::load_firmware_images(&test.config, &test.config_dir).unwrap();
    let info = process::generate_info(&loaded, &test.output_dir).unwrap();
    save_info(&info, &test.output_dir).unwrap();
}
