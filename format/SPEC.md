# TOON Format Specification (Draft)

## Versioning

- Current version: `1`
- The first byte of an encoded token is `FORMAT_VERSION`.

## Type Markers

Markers are defined in [format/src/spec/constants.rs](format/src/spec/constants.rs).

- `TYPE_NULL` = `0x00`
- `TYPE_BOOL_FALSE` = `0x01`
- `TYPE_BOOL_TRUE` = `0x02`
- `TYPE_INT64` = `0x10`
- `TYPE_F64` = `0x11`
- `TYPE_STRING` = `0x20`
- `TYPE_ARRAY` = `0x30`
- `TYPE_OBJECT` = `0x31`

## Encoding

This section will be completed once the encoder/decoder are implemented.
