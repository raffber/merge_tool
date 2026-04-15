use std::fs::{self, File};
use std::io::Write;
use std::iter::repeat;
use std::path::{Path, PathBuf};

use byteorder::{ByteOrder, LittleEndian};
use chrono::{DateTime, Utc};
use merge_tool::app_package::AppPackage;
use merge_tool::btl_trailer;
use merge_tool::config::{AddressRange, Config, DeviceConfig};
use merge_tool::crc::crc32;
use merge_tool::ed25519;
use merge_tool::firmware::Firmware;
use merge_tool::header::Header;
use merge_tool::intel_hex;
use merge_tool::process;
use serial_test::serial;
use sha2::{Digest, Sha512};

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
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        IntegrationTest {
            config,
            output_dir,
            config_dir,
        }
    }
}

#[test]
#[serial]
fn merge() {
    let test = IntegrationTest::new();
    let config = test.config;
    let config_dir = test.config_dir;
    let loaded = process::load_firmware_images(&config, &config_dir, None).unwrap();
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

    process::save_merged_firmware_images(&fws, &test.output_dir).unwrap()
}

#[test]
#[serial]
fn script() {
    let test = IntegrationTest::new();
    let loaded = process::load_firmware_images(&test.config, &test.config_dir, None).unwrap();
    let script = process::create_script(&loaded).unwrap();
    process::save_script(&script, &loaded, &test.output_dir).unwrap();
}

#[test]
#[serial]
fn info() {
    let test = IntegrationTest::new();
    let loaded = process::load_firmware_images(&test.config, &test.config_dir, None).unwrap();
    let info = process::generate_info(&loaded, &test.output_dir).unwrap();
    process::save_info(&info, &test.output_dir).unwrap();
}

#[test]
#[serial]
fn bundle() {
    let test = IntegrationTest::new();

    let loaded =
        process::load_firmware_images(&test.config, &test.config_dir, Some(&test.config_dir))
            .unwrap();

    let script = process::create_script(&loaded).unwrap();
    process::save_script(&script, &loaded, &test.output_dir).unwrap();

    let fws = process::merge_all(&loaded).unwrap();
    process::save_merged_firmware_images(&fws, &test.output_dir).unwrap();

    process::save_hex_images(&loaded, &test.output_dir).unwrap();

    let info = process::generate_info(&loaded, &test.output_dir).unwrap();
    process::save_info(&info, &test.output_dir).unwrap();

    let package = AppPackage::from_loaded_firmware_images(loaded.config.product_id, &loaded);
    process::save_app_package(&package, &test.output_dir, &loaded.app_package_file_name).unwrap();

    let bundle_output_dir = test.output_dir.join("bundle");
    process::bundle(&test.output_dir.join("info.json"), &bundle_output_dir, true).unwrap();
}

/// Ed25519 image layout used in these tests:
///   [0..64]   64-byte signature  (written by sign())
///   [64..68]  CRC32              (covers bytes 68..image_len)
///   [68..100] Firmware header    (HEADER_OFFSET = 68)
///   [100..256] Firmware code
/// The signature covers SHA-512([64..image_len]).
const IMG_SIZE: usize = 256;
const HEADER_OFFSET: u64 = 68;
const CRC_OFFSET: usize = 64;

fn make_signed_fw_image() -> Firmware {
    // Fill with a recognisable non-0xFF pattern so image_length() == IMG_SIZE.
    let data: Vec<u8> = (0..IMG_SIZE)
        .map(|i| (i as u8).wrapping_mul(3) | 0x01)
        .collect();

    let mut fw = Firmware::new(
        AddressRange::new(0, IMG_SIZE as u64),
        DeviceConfig::default(),
        data,
    )
    .unwrap();

    // Write minimal header fields (version, length).
    {
        let mut header = Header::new(&mut fw, HEADER_OFFSET).unwrap();
        header.set_major_version(1);
        header.set_minor_version(2);
        header.set_patch_version(3);
        header.set_length(IMG_SIZE as u32);
    }

    // Compute and store the CRC over [CRC_OFFSET+4 .. image_len].
    let image_len = fw.image_length();
    let crc = crc32(&fw.data[CRC_OFFSET + 4..image_len]);
    fw.write_u32(CRC_OFFSET, crc);

    fw
}

#[test]
fn ed25519_sign_and_verify() {
    use ed25519_dalek::{Signer, SigningKey};

    let mut fw = make_signed_fw_image();

    // Generate two independent key pairs to exercise key-table lookup.
    let private_key_1 = ed25519::generate_private_key();
    let public_key_1 = ed25519::public_key_bytes(&private_key_1);
    let private_key_2 = ed25519::generate_private_key();
    let public_key_2 = ed25519::public_key_bytes(&private_key_2);

    // Simulated bootloader key table: key_id -> public_key.
    let key_table: Vec<(u32, [u8; 32])> = vec![
        (crc32(&public_key_1), public_key_1),
        (crc32(&public_key_2), public_key_2),
    ];

    // Write key_id into the header (as configure_header does in production).
    let key_id_1 = crc32(&public_key_1);
    {
        let mut header = Header::new(&mut fw, HEADER_OFFSET).unwrap();
        header.set_key_id(key_id_1);
    }

    // --- Sign with key_1 ---
    ed25519::sign(&mut fw, &private_key_1).unwrap();

    // --- Stored signature must equal Ed25519(SHA512(image payload)) ---
    let image_len = fw.image_length();
    let mut sha = Sha512::new();
    Digest::input(&mut sha, &fw.data[CRC_OFFSET..image_len]);
    let digest = sha.result();
    let expected_signature = SigningKey::from_bytes(&private_key_1).sign(&digest);
    assert_eq!(&fw.data[..64], &expected_signature.to_bytes());

    // --- key_id in header must equal CRC32(public_key_1) ---
    let key_id = {
        let header = Header::new(&mut fw, HEADER_OFFSET).unwrap();
        header.key_id()
    };
    assert_eq!(key_id, crc32(&public_key_1));

    // --- Look up the public key using key_id (as a bootloader would) ---
    let found_pub_key = key_table
        .iter()
        .find(|(id, _)| *id == key_id)
        .map(|(_, k)| k)
        .expect("key_id not found in table");

    // --- Signature verifies with the correct key ---
    ed25519::verify(&fw, found_pub_key).unwrap();

    // --- key_2 must NOT verify a signature made by key_1 ---
    assert!(ed25519::verify(&fw, &public_key_2).is_err());

    // --- Any byte modification must invalidate the signature ---
    fw.data[150] ^= 0xFF;
    assert!(ed25519::verify(&fw, found_pub_key).is_err());
}

#[test]
#[serial]
fn btl_trailer_is_written_to_merged_image() {
    let mut test = IntegrationTest::new();

    // Enable the BTL trailer on the first firmware image only.
    test.config.images[0].btl_trailer = true;

    let loaded = process::load_firmware_images(&test.config, &test.config_dir, None).unwrap();
    let merged = process::merge_all(&loaded).unwrap();

    // The merged image spans both bootloader and application regions.
    // The bootloader occupies the first 256 bytes of the merged image
    // (btl_address = 0xAA00..0xAB00, app_address = 0xAB00..0xAC00).
    let merged_fw = &merged.images[0].0;

    // The last 24 bytes of the BTL region (bytes 232-255 of the merged data)
    // must contain the trailer.
    let btl_size = 256usize; // btl range is 256 bytes wide
    let trailer_start = btl_size - btl_trailer::TRAILER_TOTAL_SIZE;

    // Check magic.
    assert_eq!(
        &merged_fw.data[trailer_start..trailer_start + 4],
        &btl_trailer::TRAILER_MAGIC,
        "trailer magic not found in merged image"
    );

    // Check version.
    assert_eq!(
        merged_fw.data[trailer_start + 4],
        btl_trailer::TRAILER_VERSION
    );

    // Verify the trailer checksum covers the content correctly.
    let n = btl_size;
    let content_crc = crc32(&merged_fw.data[trailer_start..trailer_start + 16]);
    let b4: [u8; 4] = [
        merged_fw.data[n - 4],
        merged_fw.data[n - 3],
        merged_fw.data[n - 2],
        merged_fw.data[n - 1],
    ];
    let stored_crc = u32::from_le_bytes(b4);
    assert_eq!(content_crc, stored_crc, "trailer checksum mismatch");

    // Verify that the second firmware image (btl_trailer = false) has no trailer magic
    // at the same position.
    let merged_fw2 = &merged.images[1].0;
    assert_ne!(
        &merged_fw2.data[trailer_start..trailer_start + 4],
        &btl_trailer::TRAILER_MAGIC,
        "trailer should not be present when btl_trailer is false"
    );
}
