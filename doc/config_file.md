# Config File Format

The config file is a standard JSON. An minimal example:

```json
{
    "product_name": "FooBar",
    "blocking": true,
    "images": [
        {
            "btl_path": "btl/btl.hex",
            "app_path": "app/app.hex",
            "app_address": {"begin": 32768, "end": 65636},
            "btl_address": {"begin": 0, "end": 32768},
            "write_data_size": 32,
            "hex_file_format": "IntelHex",
            "device_config": {
                "word_addressing": false,
                "endianness": "Little",
                "page_size": 1024
            },
        }
    ]
}
```

Note: Paths are relative to the location of the config file.
Typically you use your build tool to copy the config file to the build folder and run the `merge_tool` from there.

## Mandatory Options

* `"product_name": "Nimbus2000"` - The name of your product. This will be inserted into the metadata of the bootload script.
* `"images": [ .... array of image configs .... ]` - The configuration of the images to be inserted into the bootload script
* `"images[k].btl_path": "path/to/btl.hex"` - The path to the hex file of the bootloader, relative to the location of the configuration file
* `"images[k].app_path": "path/to/app.hex"` - The path to the hex file of the application, relative to the location of the configuration file
* `"images[k].btl_address": {"begin": 0, "end": 1024}` - The address range where the bootloader is located in flash. `end` points 1 past the last byte to be included. Addresses must be page aligned.
* `"images[k].app_address": {"begin": 0, "end": 1024}` - The address range where the application is located in flash. `end` points 1 past the last byte to be included. Addresses must be page aligned.
* `"images[k].hex_file_format": "IntelHex"` - Defines the hex file format of the bootloader and application hex file. Either "IntelHex" or "SRecord".
* `"images[k].device_config.page_size": 1024` - Size of a flash page in the device.

## Additional Options

* `"product_id": 2518` - Allows overriding the product id. If not specified, uses the value as extracted from the firmware header.
* `"major_version": 2` - Allows overriding the major firmware version. If not specified, uses the value as extracted from the firmware header.
* `"time_state_transition": 123` - The time in milliseconds to wait between changing between state in the bootloader state machine. Default to 0.
* `"btl_version": 2` - Allows specifing the "bootloader version" field of the firmware validation data. Default to 1.
* `"use_backdoor": true` - Creates a bootload script which skips the validity check. Default to false.
* `"blocking": false` - In the script file, "query" commands are emitted for each flash write.
    Usually this means all sleep times are set to 0.
    However, this implies a handshake happening for each transaction.
    Depedning on the underlying communication protocol this may be slow.

## Additional Image Config Options

* `"fw_id" : 1` - Override the firmware id specified in the firmware header
* `"version.minor" : 1` - Override the minor firmware version specified in the firmware header
* `"version.build" : 2` - Override the minor firmware version specified in the firmware header
* `"write_data_size": 32` - The amount of data to be sent in each transaction. Defaults to `16`.
* `"include_in_script": false` - Allows creating a script file where this firmware image is not included. Default to `true`.
* `"header_offset": 4` - Allows specifying the offset of the firmware header in the application image.
    By default the firmware is placed after the 32-bit image CRC, hence the default offset is `4`.
* `"device_config.word_addressing` - Defines whether 16-bit words are used for addressing.
    This is specific to TIs C2000 architecture. Default to `false`.
* `"device_config.endianness` - Either "Big" or "Little". Default to "Little".
* `"timings.data_send": 10` - Inserts a delay between each data package. In milliseconds.
* `"timings.crc_check": 10` - Inserts a delay time after issuing the end of the data transmission. In milliseconds.
* `"timings.data_send_done": 10` - Inserts a delay time after finishing data transmission. In milliseconds.
* `"timings.leave_btl": 10` - Inserts a delay time after leaving the bootloader command. In milliseconds.
* `"timings.erase_time": 10` - Inserts a delay time after issuing an erase command. In milliseconds.
