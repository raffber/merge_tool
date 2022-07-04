# Flash Layout

## CRC Checksum

In order to allow the bootloader check the validity of the application image, a CRC32 is prepended to the data of the application image.
This CRC32 needs to be placed as the first 4-bytes of the application image.

## Firmware Header

Each application should embed a *firmware header* in it image.
This is a section at a defined flash location containing meta-information about the image.
The merge tool then extracts this meta information when post-processing the bootloader and application images.

The memory location of the firmware header is specified in the config file with the key `images[k].header_offset`.
The `header_offset` default to 4, hence it follows immedately after the 32-bit CRC of the application image.
