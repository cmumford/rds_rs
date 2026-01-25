.PHONY: build
build:
	cargo build

.PHONY: test
test:
	cargo run --example decode_rds_spy_log third_party/rds-spy-logs/Austria/A540_-_2021-07-26_19-08-06.spy

.PHONY: test-all
test-all:
	cargo run --example decode_rds_spy_log third_party/rds-spy-logs
