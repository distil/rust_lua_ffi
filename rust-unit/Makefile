all: build-debug test-debug

clean:
	cargo clean
build-debug:
	PATH=${PATH}:../lua-c-ffi-marshalling/target/debug/ cargo build
build-release:
	PATH=${PATH}:../lua-c-ffi-marshalling/target/release/ cargo build --release
test-debug: build-debug
	cargo test
test-release: build-release
	cargo test --release
