use merge_tool::config::{AddressRange, Config, FwConfig, DeviceConfig};
use merge_tool::intel_hex;
use merge_tool::process;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;
use merge_tool::crc::crc32;
use std::iter::repeat;

fn save_hex(path: &str, data: &[u8], range: &AddressRange) {
    let serialized = intel_hex::serialize(false, range, data);
    File::create(path).unwrap().write_all(serialized.as_bytes()).unwrap();
}

fn create_test_data() {
    let mut data: Vec<_> = (1u8..0x50).collect();

    save_hex("tests/btl_f1.hex", &data.clone(), &AddressRange::new(0xAA00, 0xAAFF));
    save_hex("tests/btl_f2.hex", &data.clone(), &AddressRange::new(0xAA00, 0xAAFF));

    data[4 + 2] = 1; // firmware id
    data[4 + 4] = 3; // major
    data[4 + 6] = 5; // minor
    data[4 + 8] = 4; // build
    save_hex("tests/app_f1.hex", &data.clone(), &AddressRange::new(0xAB00, 0xABFF));

    data[4 + 2] = 2; // firmware id
    data[4 + 4] = 3; // major
    data[4 + 6] = 8; // minor
    data[4 + 8] = 7; // build
    save_hex("tests/app_f2.hex", &data.clone(), &AddressRange::new(0xAB00, 0xABFF));
}

#[test]
fn merge() {
    create_test_data();
    let config_path = Path::new("tests/test.gctmrg");
    let config_dir = Config::get_config_dir(config_path).unwrap();
    let mut config = Config::load_from_file(config_path).unwrap();
    let fws = process::merge_all(&mut config, &config_dir).unwrap();

    // examine F1 firmware
    let fw = &fws[0];
    assert_eq!(fw.data[256 + 6], 1);
    assert_eq!(fw.config.page_size, 64);
    assert_eq!(fw.range.begin, 0xAA00);
    assert_eq!(fw.range.end, 0xABFF);
    assert_eq!(fw.image_length(), 256 + 128); // full btl and first 128bytes of app (2 pages)

    // reconstruct app data
    let mut data: Vec<_> = (1u8..0x50).collect();
    data.extend(repeat(0xFF).take(128 - 0x50 + 1));
    assert_eq!(data.len(), 128);
    data[4 + 2] = 1; // firmware id
    data[4 + 4] = 3; // major
    data[4 + 6] = 5; // minor
    data[4 + 8] = 4; // build

    // TODO: ....
    // // compute and compare CRC
    // let ref_crc = crc32(&data[4..]);
    // let crc = ((fw.read_u16(256 + 2) as u32) << 16) | (fw.read_u16(256 + 0) as u32);
    // assert_eq!(ref_crc, crc);

    let output_dir = config_dir.join("out");
    create_dir_all(&output_dir).unwrap();
    process::write_fws(&config, &fws, &output_dir).unwrap();
}
