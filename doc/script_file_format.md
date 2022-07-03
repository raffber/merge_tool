# Script File Format

In order to offload the implementation of certain applications from a customer system, such as the firmware update process, we define a simple script file format.
This script file format is simple to parse and suitable for parsing on bare-metal embedded systems.
The script consists of a list of commands that can be executed without jumps. Each command has 1 or 2 arguments.

| Command | Argument 1 | Argument 2 |
| ------- | ---------- | ---------- |
| Header  | Metadata (ASCII) | |
| Write   | Data to be sent (byte array) | |
| Query   | Data to be sent (byte array) | Expected Answer |
| SetTimeOut | Timeout in ms (uint32_t) | |
| Log | Log message (ASCII) | |
| SetError | Error message (ASCII) | |
| ReportProgress | Progress (uint8_t, 0 to 255) | |
| Checksum | SHA-256 checksum of the script file contents | |

The script file always starts with a `Header` command and ends with a `Checksum` command.

## Execution State Machine

The executing state machine needs to keep track of 2 state-variables:

* The currently set wait timeout. Initialized as zero.
* The error message to be printed in case of an error. Initialized as empty string.

The commands shall execute the following action:

| Command | Action |
| ------- | ---------- |
| Header | No action is taken. Data may be used for meta-information  |
| Write | Send a DDP request without requesting a response. Should the transmission fail an error condition is entered.  After executing this command, the executor shall sleep for the time setup in the current execution state. |
| Query  | Send a DDP request and request a response. The returned data is compared to the Rx data field of the command. If the response and the field do not match an error condition is entered.  After executing this command, the executor shall sleep for the time setup in the current execution state.  |
| SetTimeOut | Update the timeout field of the execution-state |
| Log | Print or record a log message. |
| SetError | Update the error message field of the execution-state |
| ReportProgress | Informs the executor about the progress in executing the script |
| Checksum | Shall be processed before executing the script. The definition of how to calculate the checksum depends on the serialization format.|

## Text Representation

This section defines a simple serialization format which is to parse on microcontrollers as well as with high-level programming languages.
Commands are encoded one per line:

* Each line starts with a “:” (colon) as a delimiter
* The remaining data is encoded in a HEX encoded byte array
* The first byte of this byte array encodes the command
* The subsequent data bytes encoded the arguments of the command
* Line breaks such as (CR or LF) are optional and are stripped before calculating the checksum. Refer to the section below.

Note that the ASCII strings are not zero terminated. The length of the string is inferred from the data-length.
Commands are serialized as follows:

| Command | Command Code | Data |
| ----- | ---- | --- |
| Header | 0x01 | ASCII String |
| Write | 0x02  | Tx data(bytearray)  |
| Query | 0x03  | Tx length: 2 bytes little endian |
| | | Rx length: 2 bytes little endian |
| | | Tx data (bytearray) |
| | | Rx data (bytearray) |
| SetTimeOut | 0x10 | Timeout little endian uint32_t |
| Log | 0x20 | ASCII string |
| SetError | 0x21 | ASCII string |
| Report Progress | 0x22 | 1 byte: 0 to 255 for 0.0 to 1.0 |
| Checksum | 0x30  | SHA-256 checksum |

## Reference Implementation

Refer to `exapmles/script.py` for a reference implementation.
