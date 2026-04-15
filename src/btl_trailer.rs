use crate::crc::crc32;
use crate::firmware::Firmware;
use crate::Error;

pub const TRAILER_MAGIC: [u8; 4] = [0x15, 0xE0, 0x27, 0x53];
pub const TRAILER_VERSION: u8 = 1;

/// Byte length of the trailer content fields (magic + version + reserved + btl_len + btl_crc).
const TRAILER_CONTENT_SIZE: usize = 16;

/// Byte length of the two meta-data fields appended after the content
/// (trailer_length: 4 B + trailer_checksum: 4 B).
const TRAILER_META_SIZE: usize = 8;

/// Total bytes consumed at the end of the BTL buffer.
pub const TRAILER_TOTAL_SIZE: usize = TRAILER_CONTENT_SIZE + TRAILER_META_SIZE;

/// Write the bootloader trailer into the last [`TRAILER_TOTAL_SIZE`] bytes of the BTL firmware
/// data buffer.
///
/// The trailer occupies the final 24 bytes of the bootloader's address region:
///
/// ```text
/// ┌─────────────────────────┬───────────────┬──────────────────┐
/// │  content  (16 B)        │  length (4 B) │ checksum  (4 B)  │
/// └─────────────────────────┴───────────────┴──────────────────┘
/// ^                          ^               ^
/// buf_len − 24               buf_len − 8     buf_len − 4
/// ```
///
/// **Trailer content layout (16 bytes, all multi-byte values little-endian):**
///
/// | Offset | Len | Description                                    |
/// |--------|-----|------------------------------------------------|
/// | +0     |  4  | Magic `[0x15, 0xE0, 0x27, 0x53]`              |
/// | +4     |  1  | Trailer version (currently `1`)                |
/// | +5     |  3  | Reserved (zeroes)                              |
/// | +8     |  4  | Bootloader image length (u32 LE)               |
/// | +12    |  4  | CRC32 over bootloader image `[0..btl_length]` |
///
/// **Meta-data:**
/// - `buf_len − 8`: Trailer length = 16 (u32 LE)
/// - `buf_len − 4`: CRC32 over the 16-byte content (u32 LE)
///
/// Returns [`Error::BtlTooSmallForTrailer`] when the firmware buffer lacks sufficient space.
pub fn write_btl_trailer(btl: &mut Firmware) -> Result<(), Error> {
    let buf_len = btl.data.len();
    let btl_length = btl.image_length();

    if btl_length + TRAILER_TOTAL_SIZE > buf_len {
        return Err(Error::BtlTooSmallForTrailer);
    }

    let btl_crc = crc32(&btl.data[0..btl_length]);

    // Build the 16-byte content block.
    let mut content = [0u8; TRAILER_CONTENT_SIZE];
    content[0..4].copy_from_slice(&TRAILER_MAGIC);
    content[4] = TRAILER_VERSION;
    // bytes 5..8: reserved, already zero
    content[8..12].copy_from_slice(&(btl_length as u32).to_le_bytes());
    content[12..16].copy_from_slice(&btl_crc.to_le_bytes());

    let content_crc = crc32(&content);

    // Write content.
    let content_start = buf_len - TRAILER_TOTAL_SIZE;
    btl.data[content_start..content_start + TRAILER_CONTENT_SIZE].copy_from_slice(&content);

    // Write meta-data.
    let meta_start = buf_len - TRAILER_META_SIZE;
    btl.data[meta_start..meta_start + 4]
        .copy_from_slice(&(TRAILER_CONTENT_SIZE as u32).to_le_bytes());
    btl.data[buf_len - 4..buf_len].copy_from_slice(&content_crc.to_le_bytes());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AddressRange, DeviceConfig};

    /// Build a Firmware whose buffer is `buf_size` bytes (must be a multiple of
    /// `DeviceConfig::default().page_size`, i.e. 64) and whose first `code_size`
    /// bytes contain a non-0xFF pattern.
    fn make_btl(buf_size: usize, code_size: usize) -> Firmware {
        assert_eq!(buf_size % 64, 0, "buf_size must be page-aligned (64 B)");
        assert!(code_size <= buf_size);
        let mut data = vec![0xFFu8; buf_size];
        for (i, b) in data[..code_size].iter_mut().enumerate() {
            *b = (i as u8).wrapping_add(1);
        }
        Firmware::new(
            AddressRange::new(0, buf_size as u64),
            DeviceConfig::default(),
            data,
        )
        .unwrap()
    }

    fn read_u32_le(data: &[u8], offset: usize) -> u32 {
        let b: [u8; 4] = [
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ];
        u32::from_le_bytes(b)
    }

    // ── Field correctness ────────────────────────────────────────────────────

    #[test]
    fn magic_bytes_are_written() {
        let mut btl = make_btl(128, 64);
        write_btl_trailer(&mut btl).unwrap();
        let n = btl.data.len();
        let start = n - TRAILER_TOTAL_SIZE;
        assert_eq!(&btl.data[start..start + 4], &TRAILER_MAGIC);
    }

    #[test]
    fn version_field_is_one() {
        let mut btl = make_btl(128, 64);
        write_btl_trailer(&mut btl).unwrap();
        let start = btl.data.len() - TRAILER_TOTAL_SIZE;
        assert_eq!(btl.data[start + 4], TRAILER_VERSION);
    }

    #[test]
    fn reserved_bytes_are_zero() {
        let mut btl = make_btl(128, 64);
        write_btl_trailer(&mut btl).unwrap();
        let start = btl.data.len() - TRAILER_TOTAL_SIZE;
        assert_eq!(&btl.data[start + 5..start + 8], &[0u8, 0, 0]);
    }

    #[test]
    fn btl_length_field_matches_image_length() {
        let mut btl = make_btl(128, 50);
        let expected_len = btl.image_length(); // rounds up to page boundary → 64
        write_btl_trailer(&mut btl).unwrap();
        let start = btl.data.len() - TRAILER_TOTAL_SIZE;
        assert_eq!(read_u32_le(&btl.data, start + 8) as usize, expected_len);
    }

    #[test]
    fn btl_crc_field_matches_crc32_of_image() {
        let mut btl = make_btl(128, 50);
        let img_len = btl.image_length();
        let expected_crc = crc32(&btl.data[0..img_len]);
        write_btl_trailer(&mut btl).unwrap();
        let start = btl.data.len() - TRAILER_TOTAL_SIZE;
        assert_eq!(read_u32_le(&btl.data, start + 12), expected_crc);
    }

    #[test]
    fn trailer_length_meta_equals_content_size() {
        let mut btl = make_btl(128, 64);
        write_btl_trailer(&mut btl).unwrap();
        let n = btl.data.len();
        assert_eq!(
            read_u32_le(&btl.data, n - TRAILER_META_SIZE) as usize,
            TRAILER_CONTENT_SIZE
        );
    }

    #[test]
    fn trailer_checksum_meta_is_crc32_of_content() {
        let mut btl = make_btl(128, 64);
        write_btl_trailer(&mut btl).unwrap();
        let n = btl.data.len();
        let content_start = n - TRAILER_TOTAL_SIZE;
        let expected = crc32(&btl.data[content_start..content_start + TRAILER_CONTENT_SIZE]);
        assert_eq!(read_u32_le(&btl.data, n - 4), expected);
    }

    // ── Error cases ──────────────────────────────────────────────────────────

    #[test]
    fn error_when_code_leaves_no_room_for_trailer() {
        // image_length() rounds up to 128 (full buffer), leaving 0 bytes free.
        let mut btl = make_btl(128, 120);
        assert!(matches!(
            write_btl_trailer(&mut btl),
            Err(Error::BtlTooSmallForTrailer)
        ));
    }

    #[test]
    fn error_when_buffer_is_smaller_than_trailer() {
        // 64 bytes of code in a 64-byte buffer → no room at all.
        let mut btl = make_btl(64, 40);
        assert!(matches!(
            write_btl_trailer(&mut btl),
            Err(Error::BtlTooSmallForTrailer)
        ));
    }

    // ── Idempotency / full-round-trip ────────────────────────────────────────

    #[test]
    fn trailer_occupies_last_24_bytes_only() {
        let code_size = 64;
        let mut btl = make_btl(192, code_size);
        // snapshot bytes before the trailer region
        let pre_trailer: Vec<u8> = btl.data[..btl.data.len() - TRAILER_TOTAL_SIZE].to_vec();
        write_btl_trailer(&mut btl).unwrap();
        assert_eq!(
            &btl.data[..btl.data.len() - TRAILER_TOTAL_SIZE],
            pre_trailer.as_slice()
        );
    }
}
