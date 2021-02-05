BOARD := omnibusf4v3
TARGET := boards/$(BOARD)/target/thumbv7em-none-eabihf/release/$(BOARD)

ifeq ($(shell uname),Linux)
	SUDO := sudo
endif

.PHONY: $(TARGET)
boards/$(BOARD)/target/thumbv7em-none-eabihf/release/$(BOARD):
	(cd boards/$(BOARD); cargo build --release)

$(BOARD).bin: $(TARGET)
	arm-none-eabi-objcopy -O binary $(TARGET) $(BOARD).bin
	dfu-suffix -v 0483 -p df11 -a $(BOARD).bin

$(BOARD).hex: $(TARGET)
	arm-none-eabi-objcopy -O ihex $(TARGET) $(BOARD).hex

.PHONY: test
test:
	cargo test

.PHONY: clean
clean:
	(cd boards/$(BOARD); cargo clean)

.PHONY: dfu
dfu: $(BOARD).bin
	$(SUDO) dfu-util -d 0483:df11 -a 0 -s 0x08000000:leave -D $(BOARD).bin

.DEFAULT_GOAL := $(BOARD).bin
