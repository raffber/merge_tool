use merge_tool::config::{AddressRange, Config, FwConfig};
use merge_tool::intel_hex;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::Path;
use merge_tool::process;

#[test]
fn merge() {
    // create test files, with the following scripts:
    // let range = AddressRange::new(0xAA00, 0xAAFF);
    // let data: Vec<_> = (1u8..21).collect();
    // let serialized = intel_hex::serialize(false, &range, &data);
    // File::create("test.hex").unwrap().write_all(serialized.as_bytes()).unwrap();
    let config_path = Path::new("tests/test.gctmrg");
    let config_dir = Config::get_config_dir(config_path).unwrap();
    let mut config = Config::load_from_file(config_path).unwrap();
    let fws = process::merge_all(&mut config, &config_dir).unwrap();
    let output_dir = config_dir.join("out");
    create_dir_all(&output_dir).unwrap();
    process::write_fws(&config, &fws[0..1], &output_dir).unwrap();
}
