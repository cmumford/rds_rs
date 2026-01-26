RELEASE=--release
RELEAES=

.PHONY: build
build:
	cargo build $(RELEASE)

.PHONY: test
test:
	cargo run $(RELEASE) --example decode_rds_spy_log third_party/rds-spy-logs/Austria/A540_-_2021-07-26_19-08-06.spy

.PHONY: test-all
test-all:
	cargo run $(RELEASE) --example decode_rds_spy_log third_party/rds-spy-logs

.PHONY: clean
clean:
	cargo clean