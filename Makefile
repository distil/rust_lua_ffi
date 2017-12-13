rust-projects := \
	c-marshalling \
	derive-c-marshalling \
	derive-c-marshalling-library \
	derive-lua-marshalling \
	lua-c-ffi-marshalling \
	lua-marshalling \
	parser \
	rust-example \
	rust-unit

.PHONY: all
all: build test run

.PHONY: build-debug-rust
build-debug-rust:
	for project in $(rust-projects); do \
		$(MAKE) -C $$project build-debug; \
	done

.PHONY: build-release-rust
build-release-rust:
	for project in $(rust-projects); do \
		$(MAKE) -C $$project build-release; \
	done

.PHONY: test-debug-rust
test-debug-rust:
	for project in $(rust-projects); do \
		$(MAKE) -C $$project test-debug; \
	done

.PHONY: test-release-rust
test-release-rust:
	for project in $(rust-projects); do \
		$(MAKE) -C $$project test-release; \
	done

.PHONY: clean-rust
clean-rust:
	for project in $(rust-projects); do \
		$(MAKE) -C $$project clean; \
	done

lua/test/luaunit.lua:
	curl https://raw.githubusercontent.com/bluebird75/luaunit/master/luaunit.lua > lua/test/luaunit.lua

.PHONY: luaunit
luaunit: lua/test/luaunit.lua

.PHONY: build-debug-example-lua
build-debug-example-lua: build-debug-rust
	mkdir -p lua/output
	LD_LIBRARY_PATH=rust-example/target/debug/ \
	luajit lua/bootstrap.lua rust_example > lua/output/rust-example.lua

.PHONY: build-release-example-lua
build-release-example-lua: build-release-rust
	mkdir -p lua/output
	LD_LIBRARY_PATH=rust-example/target/release/ \
	luajit lua/bootstrap.lua rust_example > lua/output/rust-example.lua

.PHONY: build-debug-unit-lua
build-debug-unit-lua: build-debug-rust
	mkdir -p lua/output
	LD_LIBRARY_PATH=rust-unit/target/debug/ \
	luajit lua/bootstrap.lua rust_unit > lua/output/rust-unit.lua

.PHONY: build-release-unit-lua
build-release-unit-lua: build-release-rust
	mkdir -p lua/output
	LD_LIBRARY_PATH=rust-unit/target/release/ \
	luajit lua/bootstrap.lua rust_unit > lua/output/rust-unit.lua

.PHONY: build-debug-lua
build-debug-lua: build-debug-example-lua build-debug-unit-lua

.PHONY: build-release-lua
build-release-lua: build-release-example-lua build-release-unit-lua

.PHONY: test-debug-lua
test-debug-lua: build-debug-rust build-debug-lua luaunit
	LD_LIBRARY_PATH=rust-unit/target/debug/ \
	LUA_PATH="lua/?.lua;lua/output/?.lua;lua/test/?.lua;;" \
	luajit lua/test/run.lua

.PHONY: test-release-lua
test-release-lua: build-release-rust build-release-lua luaunit
	LD_LIBRARY_PATH=rust-unit/target/release/ \
	LUA_PATH="lua/?.lua;lua/output/?.lua;lua/test/?.lua;;" \
	luajit lua/test/run.lua

.PHONY: run-debug-lua
run-debug-lua: build-debug-rust build-debug-lua
	LD_LIBRARY_PATH=rust-example/target/debug/ \
	LUA_PATH="lua/?.lua;lua/output/?.lua;;" \
	luajit lua/example.lua

.PHONY: run-release-lua
run-release-lua: build-release-rust build-release-lua
	LD_LIBRARY_PATH=rust-example/target/release/ \
	LUA_PATH="lua/?.lua;lua/output/?.lua;;" \
	luajit lua/example.lua

.PHONY: clean-lua
clean-lua:
	rm -rf lua/output
	rm -f lua/test/luaunit.lua

.PHONY: build-debug
build-debug: build-debug-rust build-debug-lua

.PHONY: build-release
build-release: build-release-rust build-release-lua

.PHONY: build
build: build-debug

.PHONY: test-debug
test-debug: test-debug-rust test-debug-lua

.PHONY: test-release
test-release: test-release-rust test-release-lua

.PHONY: test
test: test-debug

.PHONY: clean
clean: clean-rust clean-lua

.PHONY: run
run: run-debug-lua
