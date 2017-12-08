all: build test run

build-debug-rust:
	make -C c-marshalling build-debug
	make -C derive-c-marshalling build-debug
	make -C derive-c-marshalling-library build-debug
	make -C derive-lua-marshalling build-debug
	make -C lua-c-ffi-marshalling build-debug
	make -C lua-marshalling build-debug
	make -C parser build-debug
	make -C rust-example build-debug
	make -C rust-unit build-debug

build-release-rust:
	make -C c-marshalling build-release
	make -C derive-c-marshalling build-release
	make -C derive-c-marshalling-library build-release
	make -C derive-lua-marshalling build-release
	make -C lua-c-ffi-marshalling build-release
	make -C lua-marshalling build-release
	make -C rust-example build-release
	make -C rust-unit build-release

test-debug-rust:
	make -C c-marshalling test-debug
	make -C derive-c-marshalling test-debug
	make -C derive-c-marshalling-library test-debug
	make -C derive-lua-marshalling test-debug
	make -C lua-c-ffi-marshalling test-debug
	make -C lua-marshalling test-debug
	make -C parser test-debug
	make -C rust-example test-debug
	make -C rust-unit test-debug

test-release-rust:
	make -C c-marshalling test-release
	make -C derive-c-marshalling test-release
	make -C derive-c-marshalling-library test-release
	make -C derive-lua-marshalling test-release
	make -C lua-c-ffi-marshalling test-release
	make -C lua-marshalling test-release
	make -C parser test-release
	make -C rust-example test-release
	make -C rust-unit test-release

clean-rust:
	make -C c-marshalling clean
	make -C derive-c-marshalling clean
	make -C derive-c-marshalling-library clean
	make -C derive-lua-marshalling clean
	make -C lua-c-ffi-marshalling clean
	make -C lua-marshalling clean
	make -C parser clean
	make -C rust-example clean
	make -C rust-unit clean

lua/test/luaunit.lua:
	curl https://raw.githubusercontent.com/bluebird75/luaunit/master/luaunit.lua > lua/test/luaunit.lua

luaunit: lua/test/luaunit.lua

build-debug-example-lua: build-debug-rust
	mkdir -p lua/output
	LD_LIBRARY_PATH=rust-example/target/debug/ luajit lua/bootstrap.lua rust_example > lua/output/rust-example.lua
build-release-example-lua: build-release-rust
	mkdir -p lua/output
	LD_LIBRARY_PATH=rust-example/target/release/ luajit lua/bootstrap.lua rust_example > lua/output/rust-example.lua

build-debug-unit-lua: build-debug-rust
	mkdir -p lua/output
	LD_LIBRARY_PATH=rust-unit/target/debug/ luajit lua/bootstrap.lua rust_unit > lua/output/rust-unit.lua
build-release-unit-lua: build-release-rust
	mkdir -p lua/output
	LD_LIBRARY_PATH=rust-unit/target/release/ luajit lua/bootstrap.lua rust_unit > lua/output/rust-unit.lua

build-debug-lua: build-debug-example-lua build-debug-unit-lua
build-release-lua: build-release-example-lua build-release-unit-lua

test-debug-lua: build-debug-rust build-debug-lua luaunit
	LD_LIBRARY_PATH=rust-unit/target/debug/ \
	LUA_PATH="lua/?.lua;lua/output/?.lua;lua/test/?.lua;;" \
	luajit lua/test/run.lua
test-release-lua: build-release-rust build-release-lua luaunit
	LD_LIBRARY_PATH=rust-unit/target/release/ \
	LUA_PATH="lua/?.lua;lua/output/?.lua;lua/test/?.lua;;" \
	luajit lua/test/run.lua

run-debug-lua: build-debug-rust build-debug-lua
	LD_LIBRARY_PATH=rust-example/target/debug/ \
	LUA_PATH="lua/?.lua;lua/output/?.lua;;" \
	luajit lua/example.lua
run-release-lua: build-release-rust build-release-lua
	LD_LIBRARY_PATH=rust-example/target/release/ \
	LUA_PATH="lua/?.lua;lua/output/?.lua;;" \
	luajit lua/example.lua

clean-lua:
	-rm lua/output/*
	-rm lua/test/luaunit.lua

build-debug: build-debug-rust build-debug-lua
build-release: build-release-rust build-release-lua
build: build-debug

test-debug: test-debug-rust test-debug-lua
test-release: test-release-rust test-release-lua
test: test-debug

clean: clean-rust clean-lua

run: run-debug-lua
