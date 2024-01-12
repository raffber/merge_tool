# Merge And Software Release Tool

[![Rust Stable](https://github.com/raffber/merge_tool/actions/workflows/ci.yml/badge.svg)](https://github.com/raffber/merge_tool/actions/workflows/ci.yml)

This tool allows implementing a bare-metal firmware release process.

The tool takes 2 hex files as inputs: An application and a bootloader. Then it allows producing several artifacts out of them:

* A merged hex file containing both bootloader and application (typically used for production)
* A script file for updating the application
* A JSON file capturing release information about the image

As an extension of above features, it supports systems consisting of several "nodes" (== individual MCUs in a system) connected to the same communication bus. Thus, several firmware images can be merged into
one bootload file. The node receiving the images is responsible for distributing the data to the right nodes within the system.

The tool also supports reading or writing meta-data from/to the hex files.
This may be used to either configure the release process directly from the source code of the application, or, vice-versa include meta information into binary image which could then be used by the application during runtime.

## Command Line Interface

To configure the merge tool, a json config file is used. Let's assume it's called `config.json`.

To run a merge operation, i.e. merge application and booloader hex files into one, use:

```sh
./merge_tool -c config.json merge
```

To create a script file for running the updating the firmware, use:

```sh
./merge_tool -c config.json script
```

To create a JSON info file, use:

```sh
./merge_tool -c config.json info
```

The output directory may be defined with `-o <output-directory>`. If not otherwise specified, the output directory is the current working directory. For more information, call `./merge_tool --help`.

## Firmware Meta Information

The release process associates meta data to firmware images:

* Version numbers: `<major>.<minor>.<patch>`
* A product ID: An arbitrary 16-bit number which is used to uniquely identify the product to which the firmware belongs to.
* An 8-bit `node_id`: Allows specifying an MCU within a system containing multiple MCUs. For a system with a single node, this is set to 1.
* An 8-bit `bootloader_version` tag. This additional tag may be considered as an arbitrary 16-bit meta data field without pre-defined meaning. But, as the name suggests, it could be used to version the bootloader protocol.

In multi-node system, the major version and the product ID is enforced to be the same across all nodes. This follows semantic versioning.

## More Documentation

* To get an overview of all the features and how to configure the merge tool, take a look at the [configuration file](doc/config_file.md).
* To understand the communication protocol, checkout the transport layer agnostic [communication protocol description](doc/ddp_protocol.md).
* To implement a firmware update / bootloader take a look at the [bootloader protocol](doc/bootload_protocol.md).
* Also, take care about where the data should be placed in flash and what meta-information you encode in the flash. This is described in [flash layout](doc/flash_layout.md).
* Last but not least, to easily deploy your firmware update process in customer systems, consider using a [simple script file format](doc/script_file_format.md).

## Things this tool does not do

This tool is only inteded to be used where the distribution process of the firmware can be trusted or an external tool is used to establish trust into distribution process.

* It does not provide a cyrptographic signature with the image. Just a CRC32.
* It does not encrypt the software.

Pull requests to implement above features are welcome.

## License

Licensed under either of

* Apache License, Version 2.0, (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
