# Bootload Protocol

By default the bootload protocol is placed on the DDP endpoint `0x10`.

## Request and Response Format

### Request Format

| Byte `0` | Byte `1` | Bytes `2` to `n` |
| ------ | ------ | ---------------- |
| Node-ID | Command | Command Dependent Data |

### Response Format

| Byte `0` | Byte `1` | Bytes `2`  |
| ------ | ------ | ---------- |
| Node-ID | State | Error Code |

Where:

* THe Node-ID defines which MCU is update in case of a multi-MCU system. If a single MCU is updated, this value should be 1.
* Command, State and Error Codes are defined in the table below
* Note that the error code is latched when the first error occurs and can only be cleared by sending a *RESET* command
    Thus, the error code does not necessarily represent the validity of the request frame.

| Updater Command | Code | Description |
| --- | --- | --- |
| *NONE* | `0x00` | Performs no action. Allows reading back the state. |
| *RESET* | `0x0` | Resets the bootloader into *IDLE* mode. |
| *VALIDATE* | `0x02` | Sends validation data to validate the firmware to be updated against the version in the MCU |
| *START_TRANSMIT* | `0x03` | Tells the MCU to become ready to accept image data. For example, the MCU may erase some flash if it receives this command. |
| *DATA* | `0x04` | Send image data to the bootloader |
| *FINISH* |`0x05` | Tells the MCU that data transmission has finished. |
| *LEAVE* | `0x06` | Leave the bootloader and start the application. |

| Updater State | Code | Description |
| --- | --- | --- |
| *NOT_IN_BTL* |`0x00` |The application is not in update-mode |
| *IDLE* | `0x01` |The bootloader is idle and updating can start |
| *VALIDATED* | `0x02` |The image version has been validated and was accepted |
| *ERASING* | `0x03` |The bootloader is in process of erasing the flash
| *RX_DATA* | `0x04` |The bootloader is accepting image data
| *CHECKING_CRC* | `0x05` |The bootloader is verifying the CRC of the updated image
| *DONE* | `0x06` | The bootloader has successfully finished |
| *ERROR* | `0x07` | An error has occurred |

| Bootload Error | Code | Description |
| --- | --- | --- |
| *SUCCESS* | `0x00` | No error has occurred |
| *UNEXPECTED_CMD* | `0x01` | The command was not expected in the current state |
| *INVALID_CMD* | `0x02` | The command was invalid |
| *INVALID_FRAME_LENGTH* | `0x03` | The frame length was not matching the expected value |
| *INCOMPATIBLE*  | `0x04` | The firmware images are incompatible |
| *OUT_OF_BOUNDS* | `0x05` | The data to be written was out of bounds |
| *NOT_READY* | `0x06` | The bootloader was not ready to receive data |
| *INVALID_LENGTH_IN_HEADER* | `0x07` | The firmware length encoded in the header was not valid |
| *FLASH* | `0x08` | A flash error has occurred |
| *INVALID_CRC* | `0x09` | The checksum verification of the image has failed |

## State Machine

The following diagrams shows the state machine of the bootloader and how to transition between states.
Note that a *RESET* command can by sent at anytime. It will reset the bootloader back into *IDLE* state.

![Bootlader State Machine](btl.drawio.png)

## Command Description and Data Format

| Command | Data | Length | Description |
| --- | --- | --- | --- |
| *RESET* | | | Resets the state machine back to IDLE.  In case the MCU is not in bootload mode (NOT_IN_BTL), it enters bootload mode. |
| *VALIDATE* | Product ID | 2 bytes | The 16bit product ID encoded in little endian |
| | Major Version | 2 bytes | The 16bit major version encoded in little endian |
| | BTL Version | 1 byte | The version of the bootloader / protocol|
| *START_TRANSMIT* | | | Trigger the MCU to erase its flash. Once the flash has been erased, the bootloader state machine will enter the RX_DATA state and is ready to receive data. |
| *DATA* | Data offset | 4 bytes | Address offset in the firmware image |
| | Image data | 16 bytes (configurable) | Image data at the given offset address |
| *FINISH* | | | Tells the MCU that writing the image has finished and that the resulting image should be checked for validity. The state machine will enter the CHECKING_CRC state. Upon success, it enters the DONE state.|
| *LEAVE* | | | Leaves the bootloader. If uploading the image was successful, the MCU will reboot into the new firmware image.|

### Notes

The data frame length of 16 bytes is only a default value. It can be configured using the config key `images[k].write_data_size`.
For simplicity of implementation it is recommended to stay within factors of 2.
Also consider potential requirements for flash writes and ECC codes.
