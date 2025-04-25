BINARY=count-characters
TARGET=target/release/$(BINARY)
PREFIX=/usr/local/bin

.PHONY: all build install uninstall

all:
  @echo "usage: make install"
	@echo "       make uninstall"
	@echo "       make build"

build:
	cargo build --release
	@echo "Built $(BINARY) in target/release/"

install: build
	sudo cp $(TARGET) $(PREFIX)/
	@echo "Installed $(BINARY) to $(PREFIX)/"

uninstall:
	sudo rm -f $(PREFIX)/$(BINARY)
	@echo "Uninstalled $(BINARY) from $(PREFIX)/"
