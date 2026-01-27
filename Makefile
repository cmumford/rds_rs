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

.PHONY: viewer
viewer:
	cargo run $(RELEASE) --example rds_viewer "third_party/rds-spy-logs/Italy/5218 - 2023-05-10 20-41-22.spy"

.PHONY: clean
clean:
	cargo clean