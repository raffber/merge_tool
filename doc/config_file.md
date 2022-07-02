# Config File Format

The config file is a standard JSON.

## Mandatory Options

 * `"product_name": "Nimbus2000"` - The name of your product. This will be inserted into the metadata of the bootload script.
 * `"images": [ .... array of image configs .... ]` - The configuration of the images to be inserted into the bootload script
 * `"images[k].btl_path"` - The path to the hex file of the bootloader, relative to the location of the configuration file

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
* 