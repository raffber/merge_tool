# Bootload Protocol

By default the bootload protocol is placed on the DDP endpoint `0x10`.
The firmware update process is represented as a finite state machine.

## Request and Response Format

### Request Format

| Byte 0 | Byte 1 | Bytes `2` to `n` |
| ------ | ------ | ---------------- |
| Node-ID | Command | Command Dependent Data |

### Response Format

| Byte 0 | Byte 1 | Bytes `2`  |
| ------ | ------ | ---------- |
| Node-ID | State | Error Code |

Where:

* THe Node-ID defines which MCU is update in case of a multi-MCU system. If a single MCU is updated, this value should be 1.
* Command, State and Error Codes are defined in the table below
* Note that the error code is latched when the first error occurs and can only be cleared by sending a *RESET* command
    Thus, the error code does not necessarily represent the validity of the request frame.

|