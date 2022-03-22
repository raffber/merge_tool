# Config File Format

The config file is a standard JSON.

## Mandatory Options

 * `"product_name": "Nimbus2000"` - The name of your product. This will be inserted into the metadata of the bootload script.
 * `"time_state_transition": 123` - The time in milliseconds to wait between changing between state in the bootloader state machine
 * `"images": [ .... array of image configs .... ]` - The configuration of the images to be inserted into the bootload script
 * `"images[k].btl_path"` - The path to the hex file of the bootloader, relative to the location of the configuration file

## Additional Options

```

```
