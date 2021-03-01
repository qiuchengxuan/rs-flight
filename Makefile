BOARD := omnibusf4v3
TARGET := boards/$(BOARD)/target/thumbv7em-none-eabihf/release/$(BOARD)

ifeq ($(shell uname),Linux)
	SUDO := sudo
endif

.PHONY: $(TARGET)
boards/$(BOARD)/target/thumbv7em-none-eabihf/release/$(BOARD):
	(cd boards/$(BOARD); cargo build --release)

$(BOARD).dfu: $(TARGET)
	arm-none-eabi-objcopy -O binary -j .vtable $(TARGET) firmware0.bin
	arm-none-eabi-objcopy -O binary -R .vtable $(TARGET) firmware1.bin
	scripts/dfuse-pack.py -b 0x08000000:firmware0.bin -b 0x08010000:firmware1.bin $(BOARD).dfu
	rm -f firmware0.bin firmware1.bin

.PHONY: test
test:
	cargo test

.PHONY: clean
clean:
	(cd boards/$(BOARD); cargo clean)

.PHONY: dfu
dfu: $(BOARD).dfu
	$(SUDO) dfu-util -d 0483:df11 -a 0 -D $(BOARD).dfu

.DEFAULT_GOAL := $(BOARD).dfu
