# A Library for Decoding RDS/RBDS data.

Provide a complete decode implementation of the
[RDS/RBDS](https://en.wikipedia.org/wiki/Radio_Data_System) protocol
as defined by the [RBDS Specification](docs/rbds1998.pdf).

## Examples

```sh
cargo run --test decode_rds_spy_log third_party/rds-spy-logs/Austria/A540_-_2021-07-26_19-08-06.spy
```

## Running tests

A smoke test to decode one test file:
```sh
make test
```

To run the Rust tests:
```sh
cargo test
```

To run the fuzzer tests:
```sh
make fuzz
```