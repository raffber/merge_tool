# Merge And Software Release Tool

This tool allows implementing a bare-metal firmware release process.

The tool takes 2 hex files as inputs: An application and a bootloader. Then it allows producing several artifacts out of them:

* A merged hex file
* A script file for updating the application
* An JSON info file capturing release information about the image

As an extension of above features, it supports systems consisting of several "nodes" (i.e. individual MCUs). I.e. it allows processing several pairs of firmwares.

The tool also supports reading or writing meta-data from/to the hex files. This may be used to either configure the release process directly from the source code of the application, or, vice-versa include meta information into binary image which could then be used by the application during runtime.

## Command Line Interface

To configure the merge tool, a json config file is used. Let's assume it called `config.json`.

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

* Version numbers: `<major>.<minor>.<build>`
* A product ID: An arbitrary 16-bit number which is used to uniquely identify the product to which the firmware belongs to.
* A node-id: Allows specifying an MCU within a system containing multiple MCUs.
* A 16-bit `bootloader_version` tag. This additional tag may be considered as an arbitrary 16-bit meta data field without pre-defined meaning. But, as the name suggests, it could be used to version the bootloader protocol.

In multi-node system, the major version and the product ID is enforced to be the same across all nodes.

## Application Image Layout

Some firmware meta-information is stored directly inline in the application binary data. You may want to add it directly from your source code. Refer to an example here.

## Configuration File Format

## Things this tool does not do

* It does not provide a cyrptographic signature with the image. Just a CRC32.
* It does not encrypt the software.

 Pull requests to implement above features are welcome.
