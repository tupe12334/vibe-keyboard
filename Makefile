BIN := target/release/listen

.PHONY: build install run

build:
	cargo build --release

# Run once with sudo to allow the binary to access USB without sudo afterwards.
# Sets the binary as setuid-root so it can detach the HID kernel driver.
install: build
	sudo chown root:wheel $(BIN)
	sudo chmod u+s $(BIN)
	@echo "Done. Run '$(BIN)' without sudo."

run: $(BIN)
	./$(BIN)

$(BIN):
	$(MAKE) build
