# Bootloader Trailer

A trailer containing meta information of the bootloader
can be optionally included at the end of the bootloader section.

The trailer is placed at the very end of the bootloader and grows backwards.

For the CRC32 checksum format, refer to [to the flash layout document](./flash_layout.md).
Multibyte values are encoded in little endian.

## Trailer Format

The last 8 bytes of the bootloader provide trailer meta-data

| Byte Offset | Length | Description                               |
| ----------- | ------ | ----------------------------------------- |
| Length - 4  | 4      | Trailer Checksum - CRC32 over the trailer |
| Length - 8  | 4      | Trailer Length                            |

The trailer is placed immediately before the trailer meta-data.

| Byte Offset        | Length | Description                                                 |
| ------------------ | ------ | ----------------------------------------------------------- |
| Trailer Start + 0  | 4      | [0x15, 0xE0, 0x27, 0x53]                                    |
| Trailer Start + 4  | 1      | Trailer Version. Currently 1                                |
| Trailer Start + 8  | 4      | Bootloader length                                           |
| Trailer Start + 12 | 4      | CRC32 over bootloader image from 0 to the bootloader length |

