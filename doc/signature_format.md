# Signature Format

Signed images use a 64-byte Ed25519 signature at the start of the image.

## Layout

| Byte Range   | Meaning           |
| ------------ | ----------------- |
| `0..64`      | Ed25519 signature |
| `64..68`     | CRC32             |
| `68..100`    | Firmware header   |
| `100..image` | Firmware payload  |

The exact offsets for CRC and header depend on the image configuration. The signature always occupies bytes `0..64`.

## Signed Data

The signature is computed in two steps:

1. Compute `SHA-512` over the image bytes `64..image_length`.
2. Sign that 64-byte hash with Ed25519.

In other words:

```text
signature = Ed25519.Sign(private_key, SHA512(image[64..image_length]))
```

Verification uses the same hash input and the public key selected via the header `KEY_ID` field.