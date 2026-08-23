.PHONY: all elf rustlib clean

BOARDNAME = esp32:esp32:XIAO_ESP32S3_Plus
TARGETNAME =

all: elf

elf: rustlib
	arduino-cli compile -j4 --build-property "upload.maximum_size=16384000" -b esp32:esp32:XIAO_ESP32S3_Plus arduino-code --libraries Libraries --output-dir output/

rustlib:
	cargo build --release --target xtensa-esp32s3-espidf
	cp target/xtensa-esp32s3-espidf/release/libarduino_uno_starter.a Libraries/rust_ffi/src/esp32s3/librust_linkage_test.a

flash: elf
	espflash flash arduino-code/build/esp32.esp32.XIAO_ESP32S3_Plus/arduino-code.ino.elf

monitor: elf
	espflash flash -M arduino-code/build/esp32.esp32.XIAO_ESP32S3_Plus/arduino-code.ino.elf

clean:
	cargo clean
	rm -fv Libraries/rust_ffi/src/esp32s3/librust_linkage_test.a
	rm -rfv output
	rm -rfv arduino-code/build/esp32.esp32.XIAO_ESP32S3_Plus
