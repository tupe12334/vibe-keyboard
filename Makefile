BIN := target/release/listen

.PHONY: build install run

build:
	cargo build --release

# Installs `listen` to ~/.cargo/bin. Runs as a normal user (no sudo, no setuid).
install:
	cargo install --path .

run: $(BIN)
	./$(BIN)

$(BIN):
	$(MAKE) build
