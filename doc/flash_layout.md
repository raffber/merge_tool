# Flash Layout

## CRC Checksum

In order to allow the bootloader check the validity of the application image, a CRC32 is prepended to the data of the application image.
This CRC32 needs to be placed as the first 4-bytes of the application image.
The linker files should respect this and reserve the first 4 bytes of the image.

The CRC has the following parameters:

* Init = ~(0)
* Final XOR = ~(0)
* Polynomial = 0x4C11DB7
* reflected = True
* Test string: [0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39]
* Resulting CRC:  0xCBF43926

## Firmware Header

Each application should embed a *firmware header* in it image.
This is a section at a defined flash location containing meta-information about the image.
The merge tool then extracts this meta information when post-processing the bootloader and application images.

The memory location of the firmware header is specified in the config file with the key `images[k].header_offset`.
The `header_offset` defaults to 4, hence by default it follows immedately after the 32-bit CRC of the application image.

The header is 32-byte long and is 16-bit aligned (to simplify compatibility with 16-bit word addressed architectures). It has the following layout:

```c
__attribute__((section(".app_header"))) static const uint16_t app_header[] = {
    PRODUCT_ID,         // 0
    NODE_ID,            // 2
    VERSION_MAJOR,      // 4
    VERSION_MINOR,      // 6
    VERSION_BUILD_LO,   // 8
    VERSION_BUILD_HI,   // 10
    0xFFFF,             // 12
    0xFFFF,             // 14

    0xFFFF,             // 16
    0xFFFF,             // 18
    0xFFFF,             // 20
    0xFFFF,             // 22
    0xFFFF,             // 24
    0xFFFF,             // 26
    0xFFFF,             // 28
    0xFFFF,             // 30
};
```

In addition to above fields, the merge tools embeds the image length (aligned to pages) in the header.
This may be used by the bootloader to dynamically allocate flash space based on image size.
The length is written as 32-bit integer to the bytes 12 to 16.
