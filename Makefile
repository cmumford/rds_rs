RELEASE=--release
RELEAES=

TEST_FILE="third_party/rds-spy-logs/Canada/C6A8 - 2019-05-05 09-30-11.spy"
TEST_FILE="third_party/rds-spy-logs/Sweden/EC24_-_2020-08-21_17-04-13.spy"

.PHONY: build
build:
	cargo build $(RELEASE)

.PHONY: test
test:
	RUST_BACKTRACE=full cargo run $(RELEASE) --example decode_rds_spy_log $(TEST_FILE)

.PHONY: test-all
test-all:
	cargo run $(RELEASE) --example decode_rds_spy_log third_party/rds-spy-logs

.PHONY: viewer
viewer:
	cargo run $(RELEASE) --example rds_viewer $(TEST_FILE)

.PHONY: clean
clean:
	cargo clean

.PHONY: fuzz
fuzz:
	cargo fuzz run fuzz_target_1