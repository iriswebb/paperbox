#include "driver/spi_master.h"
#include "hal/gpio_ll.h"
#include "soc/spi_reg.h"
#include <Arduino.h>
#include <Print.h>
#include <SPI.h>
#include <Wire.h>
#include <pgmspace.h>
#include <rust_ffi.h>

#define TFT_WIDTH 800
#define TFT_HEIGHT 480
#define EPD_WIDTH TFT_WIDTH
#define EPD_HEIGHT TFT_HEIGHT
#define TFT_SCLK D8
#define TFT_MISO -1
#define TFT_MOSI D10
#define TFT_CS 44     // D7
#define TFT_DC 10     // D16
#define TFT_BUSY 4    // D3
#define TFT_RST 38    // D11
#define TFT_ENABLE 43 // D6

#define SPI_FREQUENCY 10000000
#define SPI_READ_FREQUENCY 4000000

#define EPD_WIDTH 800
#define EPD_HEIGHT 480
#define TFT_WIDTH EPD_WIDTH
#define TFT_HEIGHT EPD_HEIGHT
#define EPD_COLOR_DEPTH 1

#define EPD_NOP 0xFF
#define EPD_DISPON 0x04
#define EPD_DISPOFF 0x03

#define TFT_INVON EPD_DISPON
#define TFT_INVOFF EPD_DISPOFF

const unsigned char LUT_VCOM[] = {
    0x26, 0x0F, 0x18, 0x18, 0x14, 0x01, 0x00, 0x0A, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
};
const unsigned char LUT_WW[] = {
    0x55, 0x06, 0x0C, 0x17, 0x02, 0x01, 0x2A, 0x02, 0x1C, 0x02, 0x0D,
    0x01, 0x80, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
};
const unsigned char LUT_KW[] = {
    0x55, 0x06, 0x0C, 0x17, 0x02, 0x01, 0x2A, 0x02, 0x1C, 0x02, 0x0D,
    0x01, 0x80, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
};
const unsigned char LUT_WK[] = {
    0xAA, 0x06, 0x0C, 0x17, 0x02, 0x01, 0x15, 0x02, 0x1C, 0x02, 0x0D,
    0x01, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
};
const unsigned char LUT_KK[] = {
    0xAA, 0x06, 0x0C, 0x17, 0x02, 0x01, 0x15, 0x02, 0x1C, 0x02, 0x0D,
    0x01, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
};
const unsigned char CMD_USER[] = {0X17, 0X3F, 0X3F, 0X09, 0X06, 0X16};
const unsigned char LUT_VCOM_GRAY[] = {
    0x00, 0x00, 0x06, 0x08, 0x07, 0x01, 0x00, 0x06, 0x0A, 0x0B, 0x0A,
    0x01, 0x00, 0x03, 0x03, 0x00, 0x00, 0x03, 0x00, 0x05, 0x09, 0x06,
    0x06, 0x01, 0x00, 0x02, 0x02, 0x0A, 0x0A, 0x01, 0x00, 0x0A, 0x11,
    0x06, 0x07, 0x01, 0x00, 0x02, 0x01, 0x02, 0x01, 0x01,
};
const unsigned char LUT_WW_GRAY[] = {
    0x15, 0x00, 0x06, 0x08, 0x07, 0x01, 0x54, 0x06, 0x0A, 0x0B, 0x0A,
    0x01, 0x90, 0x03, 0x03, 0x00, 0x00, 0x03, 0x2A, 0x05, 0x09, 0x06,
    0x06, 0x01, 0xAA, 0x02, 0x02, 0x0A, 0x0A, 0x01, 0x00, 0x0A, 0x11,
    0x06, 0x07, 0x01, 0x28, 0x02, 0x01, 0x02, 0x01, 0x01,
};
const unsigned char LUT_KW_GRAY[] = {
    0x2A, 0x00, 0x06, 0x08, 0x07, 0x01, 0x59, 0x06, 0x0A, 0x0B, 0x0A,
    0x01, 0x90, 0x03, 0x03, 0x00, 0x00, 0x03, 0x5A, 0x05, 0x09, 0x06,
    0x06, 0x01, 0xA8, 0x02, 0x02, 0x0A, 0x0A, 0x01, 0x45, 0x0A, 0x11,
    0x06, 0x07, 0x01, 0xA8, 0x02, 0x01, 0x02, 0x01, 0x01,
};
const unsigned char LUT_WK_GRAY[] = {
    0x16, 0x00, 0x06, 0x08, 0x07, 0x01, 0xA0, 0x06, 0x0A, 0x0B, 0x0A,
    0x01, 0x90, 0x03, 0x03, 0x00, 0x00, 0x03, 0x99, 0x05, 0x09, 0x06,
    0x06, 0x01, 0xA0, 0x02, 0x02, 0x0A, 0x0A, 0x01, 0x40, 0x0A, 0x11,
    0x06, 0x07, 0x01, 0x20, 0x02, 0x01, 0x02, 0x01, 0x01,
};
const unsigned char LUT_KK_GRAY[] = {
    0x26, 0x00, 0x06, 0x08, 0x07, 0x01, 0x6A, 0x06, 0x0A, 0x0B, 0x0A,
    0x01, 0x90, 0x03, 0x03, 0x00, 0x00, 0x03, 0x65, 0x05, 0x09, 0x06,
    0x06, 0x01, 0x50, 0x02, 0x02, 0x0A, 0x0A, 0x01, 0x10, 0x0A, 0x11,
    0x06, 0x07, 0x01, 0x10, 0x02, 0x01, 0x02, 0x01, 0x01,
};
const unsigned char CMD_USER_GRAY[] = {
    0X17, 0X3F, 0X3F, 0X07, 0X06, 0X12,
};

#define CHECK_BUSY()                                                           \
  do {                                                                         \
    delay(10);                                                                 \
    if (digitalRead(TFT_BUSY))                                                 \
      break;                                                                   \
  } while (true)

#define UC8179_USE_INTERNAL_OTP() (_uc8179_use_otp_lut)

#define UC8179_INIT_GRAY_OTP()                                                 \
  do {                                                                         \
    writecommand(0x01);                                                        \
    writedata(0x07);                                                           \
    writedata(0x07);                                                           \
    writedata(0x3F);                                                           \
    writedata(0x3F);                                                           \
    writecommand(0x06);                                                        \
    writedata(0x27);                                                           \
    writedata(0x27);                                                           \
    writedata(0x18);                                                           \
    writedata(0x17);                                                           \
    writecommand(0x04);                                                        \
    delay(100);                                                                \
    CHECK_BUSY();                                                              \
    writecommand(0x00);                                                        \
    writedata(0x1F);                                                           \
    writecommand(0x61);                                                        \
    writedata(EPD_WIDTH >> 8);                                                 \
    writedata(EPD_WIDTH & 0xFF);                                               \
    writedata(EPD_HEIGHT >> 8);                                                \
    writedata(EPD_HEIGHT & 0xFF);                                              \
    writecommand(0x50);                                                        \
    writedata(0x10);                                                           \
    writedata(0x07);                                                           \
    writecommand(0xE0);                                                        \
    writedata(0x02);                                                           \
    writecommand(0xE5);                                                        \
    writedata(0x5F);                                                           \
  } while (0)

#define EPD_WRITE_LUT()                                                        \
  do {                                                                         \
    uint16_t count;                                                            \
    CHECK_BUSY();                                                              \
    writecommand(0x20);                                                        \
    for (count = 0; count < 42; count++) {                                     \
      writedata(LUT_VCOM[count]);                                              \
    }                                                                          \
    CHECK_BUSY();                                                              \
    writecommand(0x21);                                                        \
    for (count = 0; count < 42; count++) {                                     \
      writedata(LUT_WW[count]);                                                \
    }                                                                          \
    CHECK_BUSY();                                                              \
    writecommand(0x22);                                                        \
    for (count = 0; count < 42; count++) {                                     \
      writedata(LUT_KW[count]);                                                \
    }                                                                          \
    CHECK_BUSY();                                                              \
    writecommand(0x23);                                                        \
    for (count = 0; count < 42; count++) {                                     \
      writedata(LUT_WK[count]);                                                \
    }                                                                          \
    writecommand(0x24);                                                        \
    for (count = 0; count < 42; count++) {                                     \
      writedata(LUT_KK[count]);                                                \
    }                                                                          \
  } while (0)

#define EPD_WRITE_LUT_GRAY()                                                   \
  do {                                                                         \
    uint16_t count;                                                            \
    CHECK_BUSY();                                                              \
    writecommand(0x20);                                                        \
    for (count = 0; count < 42; count++) {                                     \
      writedata(LUT_VCOM_GRAY[count]);                                         \
    }                                                                          \
    CHECK_BUSY();                                                              \
    writecommand(0x21);                                                        \
    for (count = 0; count < 42; count++) {                                     \
      writedata(LUT_WW_GRAY[count]);                                           \
    }                                                                          \
    CHECK_BUSY();                                                              \
    writecommand(0x22);                                                        \
    for (count = 0; count < 42; count++) {                                     \
      writedata(LUT_KW_GRAY[count]);                                           \
    }                                                                          \
    CHECK_BUSY();                                                              \
    writecommand(0x23);                                                        \
    for (count = 0; count < 42; count++) {                                     \
      writedata(LUT_WK_GRAY[count]);                                           \
    }                                                                          \
    writecommand(0x24);                                                        \
    for (count = 0; count < 42; count++) {                                     \
      writedata(LUT_KK_GRAY[count]);                                           \
    }                                                                          \
  } while (0)

#define EPD_INIT_FAST()                                                        \
  do {                                                                         \
    writecommand(0x01);                                                        \
    writedata(0x07);                                                           \
    writedata(CMD_USER[0]);                                                    \
    writedata(CMD_USER[1]);                                                    \
    writedata(CMD_USER[2]);                                                    \
    writedata(CMD_USER[3]);                                                    \
    writecommand(0x30);                                                        \
    writedata(CMD_USER[4]);                                                    \
    writecommand(0x82);                                                        \
    writedata(CMD_USER[5]);                                                    \
    writecommand(0x06);                                                        \
    writedata(0x17);                                                           \
    writedata(0x17);                                                           \
    writedata(0x28);                                                           \
    writedata(0x17);                                                           \
    writecommand(0x04);                                                        \
    delay(100);                                                                \
    CHECK_BUSY();                                                              \
    writecommand(0x00);                                                        \
    writedata(0x3F);                                                           \
    writecommand(0x61);                                                        \
    writedata(EPD_WIDTH >> 8);                                                 \
    writedata(EPD_WIDTH & 0xFF);                                               \
    writedata(EPD_HEIGHT >> 8);                                                \
    writedata(EPD_HEIGHT & 0xFF);                                              \
    writecommand(0x50);                                                        \
    writedata(0x10);                                                           \
    writedata(0x07);                                                           \
    EPD_WRITE_LUT();                                                           \
  } while (0)

#define EPD_INIT_GRAY()                                                        \
  do {                                                                         \
    if (!_uc8179_has_checked_otp) {                                            \
      uc8179ProbeOtpSupport();                                                 \
    }                                                                          \
    if (UC8179_USE_INTERNAL_OTP()) {                                           \
      UC8179_INIT_GRAY_OTP();                                                  \
    } else {                                                                   \
      writecommand(0x01);                                                      \
      writedata(0x07);                                                         \
      writedata(CMD_USER_GRAY[0]);                                             \
      writedata(CMD_USER_GRAY[1]);                                             \
      writedata(CMD_USER_GRAY[2]);                                             \
      writedata(CMD_USER_GRAY[3]);                                             \
      writecommand(0x30);                                                      \
      writedata(CMD_USER_GRAY[4]);                                             \
      writecommand(0x82);                                                      \
      writedata(CMD_USER_GRAY[5]);                                             \
      writecommand(0x06);                                                      \
      writedata(0x27);                                                         \
      writedata(0x27);                                                         \
      writedata(0x28);                                                         \
      writedata(0x17);                                                         \
      writecommand(0x04);                                                      \
      delay(100);                                                              \
      CHECK_BUSY();                                                            \
      writecommand(0x00);                                                      \
      writedata(0x3F);                                                         \
      writecommand(0xE3);                                                      \
      writedata(0x88);                                                         \
      writecommand(0x50);                                                      \
      writedata(0x10);                                                         \
      writedata(0x07);                                                         \
      writecommand(0x52);                                                      \
      writedata(0x00);                                                         \
      writecommand(0x61);                                                      \
      writedata(EPD_WIDTH >> 8);                                               \
      writedata(EPD_WIDTH & 0xFF);                                             \
      writedata(EPD_HEIGHT >> 8);                                              \
      writedata(EPD_HEIGHT & 0xFF);                                            \
      EPD_WRITE_LUT_GRAY();                                                    \
    }                                                                          \
  } while (0)

#define EPD_INIT_PARTIAL()                                                     \
  do {                                                                         \
    writecommand(0x00);                                                        \
    writedata(0x1F);                                                           \
    writecommand(0x04);                                                        \
    delay(100);                                                                \
    CHECK_BUSY();                                                              \
    writecommand(0xE0);                                                        \
    writedata(0x02);                                                           \
    writecommand(0xE5);                                                        \
    writedata(0x6E);                                                           \
  } while (0)

#define EPD_WAKEUP()                                                           \
  do {                                                                         \
    digitalWrite(TFT_RST, LOW);                                                \
    delay(10);                                                                 \
    digitalWrite(TFT_RST, HIGH);                                               \
    delay(10);                                                                 \
    CHECK_BUSY();                                                              \
    EPD_INIT_FAST();                                                           \
  } while (0)

#define EPD_WAKEUP_GRAY()                                                      \
  do {                                                                         \
    digitalWrite(TFT_RST, LOW);                                                \
    delay(10);                                                                 \
    digitalWrite(TFT_RST, HIGH);                                               \
    delay(10);                                                                 \
    CHECK_BUSY();                                                              \
    EPD_INIT_GRAY();                                                           \
  } while (0)

#define EPD_WAKEUP_PARTIAL()                                                   \
  do {                                                                         \
    digitalWrite(TFT_RST, LOW);                                                \
    delay(10);                                                                 \
    digitalWrite(TFT_RST, HIGH);                                               \
    delay(10);                                                                 \
    CHECK_BUSY();                                                              \
    EPD_INIT_PARTIAL();                                                        \
  } while (0);

#define EPD_SET_WINDOW(x1, y1, x2, y2)                                         \
  do {                                                                         \
    writecommand(0x50);                                                        \
    writedata(0xA9);                                                           \
    writedata(0x07);                                                           \
    writecommand(0x91);                                                        \
    writecommand(0x90);                                                        \
    writedata(x1 >> 8);                                                        \
    writedata(x1 & 0xFF);                                                      \
    writedata(x2 >> 8);                                                        \
    writedata((x2 & 0xFF) - 1);                                                \
    writedata(y1 >> 8);                                                        \
    writedata(y1 & 0xFF);                                                      \
    writedata(y2 >> 8);                                                        \
    writedata((y2 & 0xFF) - 1);                                                \
    writedata(0x01);                                                           \
  } while (0)

#define EPD_PUSH_NEW_COLORS(w, h, colors)                                      \
  do {                                                                         \
    writecommand(0x13);                                                        \
    for (int i = 0; i < w * h / 8; i++) {                                      \
      writedata(colors[i]);                                                    \
    }                                                                          \
  } while (0)

#define EPD_PUSH_NEW_GRAY_COLORS(w, h, colors)                                 \
  do {                                                                         \
    uint16_t i, j, k;                                                          \
    uint8_t temp1, temp2, temp3;                                               \
    writecommand(0x10);                                                        \
    for (i = 0; i < 48000; i++) {                                              \
      /* Read 4 input bytes = 8 pixels */                                      \
      uint8_t c0 = colors[i * 4 + 0];                                          \
      uint8_t c1 = colors[i * 4 + 1];                                          \
      uint8_t c2 = colors[i * 4 + 2];                                          \
      uint8_t c3 = colors[i * 4 + 3];                                          \
      /* Extract 8 pixels from bit5-4 and bit1-0 of each byte */               \
      uint8_t p0 = (c0 >> 4) & 0x03;                                           \
      uint8_t p1 = (c0 >> 0) & 0x03;                                           \
      uint8_t p2 = (c1 >> 4) & 0x03;                                           \
      uint8_t p3 = (c1 >> 0) & 0x03;                                           \
      uint8_t p4 = (c2 >> 4) & 0x03;                                           \
      uint8_t p5 = (c2 >> 0) & 0x03;                                           \
      uint8_t p6 = (c3 >> 4) & 0x03;                                           \
      uint8_t p7 = (c3 >> 0) & 0x03;                                           \
      /* Pack into original-style packed bytes (high 2-bit per pixel) */       \
      uint8_t packed_byte0 = (p0 << 6) | (p1 << 4) | (p2 << 2) | p3;           \
      uint8_t packed_byte1 = (p4 << 6) | (p5 << 4) | (p6 << 2) | p7;           \
                                                                               \
      temp3 = 0;                                                               \
      for (j = 0; j < 2; j++) {                                                \
        temp1 = (j == 0) ? packed_byte0 : packed_byte1;                        \
        for (k = 0; k < 4; k++) {                                              \
          temp2 = temp1 & 0xC0;                                                \
          if (temp2 == 0xC0)                                                   \
            temp3 |= 0x01;                                                     \
          else if (temp2 == 0x00)                                              \
            temp3 |= 0x00;                                                     \
          else if ((temp2 >= 0x80) && (temp2 < 0xC0))                          \
            temp3 |= 0x00;                                                     \
          else if (temp2 == 0x40)                                              \
            temp3 |= 0x01;                                                     \
          if ((j == 0 && k <= 3) || (j == 1 && k <= 2)) {                      \
            temp3 <<= 1;                                                       \
            temp1 <<= 2;                                                       \
          }                                                                    \
        }                                                                      \
      }                                                                        \
      writedata(temp3);                                                        \
    }                                                                          \
                                                                               \
    writecommand(0x13);                                                        \
    for (i = 0; i < 48000; i++) {                                              \
      uint8_t c0 = colors[i * 4 + 0];                                          \
      uint8_t c1 = colors[i * 4 + 1];                                          \
      uint8_t c2 = colors[i * 4 + 2];                                          \
      uint8_t c3 = colors[i * 4 + 3];                                          \
      uint8_t p0 = (c0 >> 4) & 0x03;                                           \
      uint8_t p1 = (c0 >> 0) & 0x03;                                           \
      uint8_t p2 = (c1 >> 4) & 0x03;                                           \
      uint8_t p3 = (c1 >> 0) & 0x03;                                           \
      uint8_t p4 = (c2 >> 4) & 0x03;                                           \
      uint8_t p5 = (c2 >> 0) & 0x03;                                           \
      uint8_t p6 = (c3 >> 4) & 0x03;                                           \
      uint8_t p7 = (c3 >> 0) & 0x03;                                           \
      uint8_t packed_byte0 = (p0 << 6) | (p1 << 4) | (p2 << 2) | p3;           \
      uint8_t packed_byte1 = (p4 << 6) | (p5 << 4) | (p6 << 2) | p7;           \
                                                                               \
      temp3 = 0;                                                               \
      for (j = 0; j < 2; j++) {                                                \
        temp1 = (j == 0) ? packed_byte0 : packed_byte1;                        \
        for (k = 0; k < 4; k++) {                                              \
          temp2 = temp1 & 0xC0;                                                \
          if (temp2 == 0xC0)                                                   \
            temp3 |= 0x01;                                                     \
          else if (temp2 == 0x00)                                              \
            temp3 |= 0x00;                                                     \
          else if ((temp2 >= 0x80) && (temp2 < 0xC0))                          \
            temp3 |= 0x01;                                                     \
          else if (temp2 == 0x40)                                              \
            temp3 |= 0x00;                                                     \
          if ((j == 0 && k <= 3) || (j == 1 && k <= 2)) {                      \
            temp3 <<= 1;                                                       \
            temp1 <<= 2;                                                       \
          }                                                                    \
        }                                                                      \
      }                                                                        \
      writedata(temp3);                                                        \
    }                                                                          \
  } while (0)

#define EPD_PUSH_OLD_COLORS(w, h, colors)                                      \
  do {                                                                         \
    writecommand(0x10);                                                        \
    for (int i = 0; i < w * h / 8; i++) {                                      \
      writedata(colors[i]);                                                    \
    }                                                                          \
  } while (0)

#define VSPI FSPI
#define SPI_MOSI_DLEN_REG(x) SPI_MS_DLEN_REG(x)
#define SUPPORT_TRANSACTIONS
#define SPI_PORT 3

#define _spi_cmd (volatile uint32_t *)(SPI_CMD_REG(SPI_PORT))
#define _spi_user (volatile uint32_t *)(SPI_USER_REG(SPI_PORT))
#define _spi_mosi_dlen (volatile uint32_t *)(SPI_MOSI_DLEN_REG(SPI_PORT))
#define _spi_w (volatile uint32_t *)(SPI_W0_REG(SPI_PORT))

#define SET_BUS_WRITE_MODE *_spi_user = SPI_USR_MOSI
#define SET_BUS_READ_MODE *_spi_user = SPI_USR_MOSI | SPI_USR_MISO | SPI_DOUTDIN

#define SPI_BUSY_CHECK while (*_spi_cmd & SPI_USR)

// Other RPi displays need a slower C->D change
#define DC_C GPIO.out_w1tc = (1 << TFT_DC)
#define DC_D                                                                   \
  GPIO.out_w1tc = (1 << TFT_DC);                                               \
  GPIO.out_w1ts = (1 << TFT_DC)
#define CS_L                                                                   \
  GPIO.out1_w1tc.val = (1 << (TFT_CS - 32));                                   \
  GPIO.out1_w1tc.val = (1 << (TFT_CS - 32))
#define CS_H                                                                   \
  GPIO.out1_w1ts.val =                                                         \
      (1 << (TFT_CS - 32)) //;GPIO.out1_w1ts.val = (1 << (TFT_CS - 32))

#define TFT_WRITE_BITS(D, B)                                                   \
  *_spi_mosi_dlen = B - 1;                                                     \
  *_spi_w = D;                                                                 \
  *_spi_cmd = SPI_USR;                                                         \
  while (*_spi_cmd & SPI_USR)                                                  \
    ;

#define tft_Write_8(C) TFT_WRITE_BITS(C, 8)
#define TFT_SPI_MODE SPI_MODE0
#define TFT_WHITE 0xFFFF /* 255, 255, 255 */

// Class functions and variables
class TFT_eSPI {

public:
  TFT_eSPI(int16_t _W = TFT_WIDTH, int16_t _H = TFT_HEIGHT);
  ~TFT_eSPI(void);

  void init(), initFromSleep(), begin();
  virtual int16_t height(void), width(void);
  void invertDisplay(bool i);   // Tell TFT to invert all displayed colours
  void writecommand(uint8_t c); // Send an 8-bit command, function resets DC/RS
  void writedata(uint8_t d);    // Send data with DC/RS set high
  inline void begin_tft_write() __attribute__((always_inline));
  inline void end_tft_write() __attribute__((always_inline));
  void initBus(void);
  void uc8179ProbeOtpSupport(void);
  void uc8179ReadTemperatureRaw(uint8_t *temp1, uint8_t *temp2);
  void uc8179ReadOtpUserData(uint16_t read_len, uint16_t data_offset,
                             uint8_t *buf, uint8_t buf_len);
  void *createSprite(int16_t width, int16_t height);
  void deleteSprite(void);
  void *setColorDepth(int8_t b);
  void drawPixel(int32_t x, int32_t y, uint32_t color);
  void drawBitmap(int16_t x, int16_t y, const uint8_t *bitmap, int16_t w,
                  int16_t h, uint16_t fgcolor, uint16_t bgcolor);
  void fillSprite(uint32_t color),
      fillRect(int32_t x, int32_t y, int32_t w, int32_t h, uint32_t color);
  uint16_t readPixelValue(int32_t x, int32_t y);
  void *callocSprite(int16_t width, int16_t height);
  uint8_t _bpp;  // bits per pixel (1, 4, 8 or 16)
  uint8_t *_img; // pointer to  1 and 8-bit sprite frame 1 or frame 2
  bool _created; // A Sprite has been created and memory reserved
  bool locked, inTransaction, lockTransaction;
  bool _booted;
  bool _uc8179_has_checked_otp;
  bool _uc8179_use_otp_lut;
  uint8_t _uc8179_temp_raw_int;
  uint8_t _uc8179_temp_raw_frac;
}; // End of class TFT_eSPI

class EPaper : public TFT_eSPI {
public:
  explicit EPaper();

  void begin(uint8_t wake = 0);
  void drawBufferPixel(int32_t x, int32_t y, uint32_t color, uint8_t bpp);
  void update();
  void updataPartial(uint16_t x, uint16_t y, uint16_t w, uint16_t h);
  void initGrayMode();
  void deinitGrayMode();
  void sleep();
  void wake();

  bool _sleep;
};

SPIClass spi = SPIClass(HSPI);

/***************************************************************************************
** Function name:           begin_tft_write (was called spi_begin)
** Description:             Start SPI transaction for writes and select TFT
***************************************************************************************/
inline void TFT_eSPI::begin_tft_write(void) {
  if (locked) {
    locked = false; // Flag to show SPI access now unlocked
    spi.beginTransaction(SPISettings(SPI_FREQUENCY, MSBFIRST, TFT_SPI_MODE));
    CS_L;
    SET_BUS_WRITE_MODE; // Some processors (e.g. ESP32) allow recycling the tx
                        // buffer when rx is not used
  }
}

/***************************************************************************************
** Function name:           end_tft_write (was called spi_end)
** Description:             End transaction for write and deselect TFT
***************************************************************************************/
inline void TFT_eSPI::end_tft_write(void) {
  if (!inTransaction) { // Flag to stop ending transaction during multiple
                        // graphics calls
    if (!locked) {      // Locked when beginTransaction has been called
      locked = true;    // Flag to show SPI access now locked
      SPI_BUSY_CHECK;   // Check send complete and clean out unused rx data
      CS_H;
      SET_BUS_READ_MODE; // In case bus has been configured for tx only
      spi.endTransaction();
    }
  }
}

/***************************************************************************************
** Function name:           TFT_eSPI
** Description:             Constructor , we must use hardware SPI pins
***************************************************************************************/
TFT_eSPI::TFT_eSPI(int16_t w, int16_t h) {
  locked = true; // Transaction mutex lock flag to ensure begin/endTranaction
  inTransaction = false; // Flag to prevent multiple sequential functions to
  lockTransaction = false;
  _booted = true; // Default attributes
  _uc8179_has_checked_otp = false;
  _uc8179_use_otp_lut = false;
  _uc8179_temp_raw_int = 0;
  _uc8179_temp_raw_frac = 0;
  _bpp = 1;
  _created = false;
  lockTransaction = true;
}

/***************************************************************************************
** Function name:           initBus
** Description:             initialise the SPI or parallel bus
***************************************************************************************/
void TFT_eSPI::initBus(void) {

  pinMode(TFT_CS, OUTPUT);
  digitalWrite(TFT_CS, HIGH); // Chip select high (inactive)

// In parallel mode and with the RP2040 processor, the TFT_WR line is handled in
// the  PIO
#if defined(TFT_WR) && !defined(ARDUINO_ARCH_RP2040) &&                        \
    !defined(ARDUINO_ARCH_MBED)
  if (TFT_WR >= 0) {
    pinMode(TFT_WR, OUTPUT);
    digitalWrite(TFT_WR, HIGH); // Set write strobe high (inactive)
  }
#endif

  pinMode(TFT_DC, OUTPUT);
  digitalWrite(TFT_DC, HIGH); // Data/Command high = data mode

  pinMode(TFT_RST, OUTPUT);
  digitalWrite(TFT_RST,
               HIGH); // Set high, do not share pin with another SPI device

#if defined(TFT_PARALLEL_8_BIT)

  // Make sure read is high before we set the bus to output
  if (TFT_RD >= 0) {
    pinMode(TFT_RD, OUTPUT);
    digitalWrite(TFT_RD, HIGH);
  }

#  if !defined(ARDUINO_ARCH_RP2040) &&                                         \
      !defined(ARDUINO_ARCH_MBED) // PIO manages pins
  // Set TFT data bus lines to output
  pinMode(TFT_D0, OUTPUT);
  digitalWrite(TFT_D0, HIGH);
  pinMode(TFT_D1, OUTPUT);
  digitalWrite(TFT_D1, HIGH);
  pinMode(TFT_D2, OUTPUT);
  digitalWrite(TFT_D2, HIGH);
  pinMode(TFT_D3, OUTPUT);
  digitalWrite(TFT_D3, HIGH);
  pinMode(TFT_D4, OUTPUT);
  digitalWrite(TFT_D4, HIGH);
  pinMode(TFT_D5, OUTPUT);
  digitalWrite(TFT_D5, HIGH);
  pinMode(TFT_D6, OUTPUT);
  digitalWrite(TFT_D6, HIGH);
  pinMode(TFT_D7, OUTPUT);
  digitalWrite(TFT_D7, HIGH);
#  endif

  PARALLEL_INIT_TFT_DATA_BUS;

#endif
}

/***************************************************************************************
** Function name:           begin
** Description:             Included for backwards compatibility
***************************************************************************************/
void TFT_eSPI::begin() { init(); }

void TFT_eSPI::initFromSleep() {

  if (!_booted)
    return;
  _booted = false;

  initBus();

#if defined(TFT_MOSI) && !defined(TFT_SPI_OVERLAP) &&                          \
    !defined(ARDUINO_ARCH_RP2040) && !defined(ARDUINO_ARCH_MBED) &&            \
    !defined(EFR32MG24B220F1536IM48)
#  if defined(NRF52840_XXAA)
  spi.setPins(TFT_MISO, TFT_SCLK, TFT_MOSI);
  spi.begin(); // nRF52 core uses setPins() instead of begin(sck, miso, mosi,
               // ss)
#  else
  spi.begin(TFT_SCLK, TFT_MISO, TFT_MOSI, -1); // This will set MISO to input
#  endif
#else
  spi.begin(); // This will set MISO to input
#endif

  lockTransaction = false;
  inTransaction = false;
  locked = true;

  pinMode(TFT_CS, OUTPUT);
  digitalWrite(TFT_CS, HIGH);

  if (TFT_DC >= 0) {
    pinMode(TFT_DC, OUTPUT);
    digitalWrite(TFT_DC, HIGH);
  }

  if (TFT_RST >= 0) {
    pinMode(TFT_RST, OUTPUT);
  }

  // This is the command sequence that initialises the  UC8179 driver

  {

    pinMode(TFT_BUSY, INPUT);
    pinMode(TFT_ENABLE, OUTPUT);
    digitalWrite(TFT_ENABLE, HIGH);
    writecommand(0x01); // POWER SETTING
    writedata(0x07);
    writedata(0x07); // VGH=20V,VGL=-20V
    writedata(0x3f); // VDH=15V
    writedata(0x3f); // VDL=-15V

    // Enhanced display drive(Add 0x06 command)
    writecommand(0x06); // Booster Soft Start
    writedata(0x17);
    writedata(0x17);
    writedata(0x28);
    writedata(0x17);

    writecommand(0x04); // POWER ON
    delay(100);

    CHECK_BUSY();

    writecommand(0X00); // PANNEL SETTING
    writedata(0x1F);    // KW-3f   KWR-2F BWROTP 0f BWOTP 1f

    writecommand(0x61); // tres
    writedata(EPD_WIDTH >> 8);
    writedata(EPD_WIDTH & 0xFF);
    writedata(EPD_HEIGHT >> 8);
    writedata(EPD_HEIGHT & 0xFF);

    writecommand(0X15);
    writedata(0x00);

    writecommand(0X50); // VCOM AND DATA INTERVAL SETTING
    writedata(0x10);
    writedata(0x07);

    writecommand(0X60); // TCON SETTING
    writedata(0x22);
  }
}
/***************************************************************************************
** Function name:           init (tc is tab colour for ST7735 displays only)
** Description:             Reset, then initialise the TFT display registers
***************************************************************************************/
void TFT_eSPI::init() {
  if (_booted) {
    initBus();

    spi.begin(TFT_SCLK, TFT_MISO, TFT_MOSI, -1); // This will set MISO to input
    lockTransaction = false;
    inTransaction = false;
    locked = true;

    pinMode(TFT_CS, OUTPUT);
    digitalWrite(TFT_CS, HIGH); // Chip select high (inactive)

    // Set to output once again in case MISO is used for DC
    pinMode(TFT_DC, OUTPUT);
    digitalWrite(TFT_DC, HIGH); // Data/Command high = data mode

    _booted = false;
    end_tft_write();
  } // end of: if just _booted

  if (TFT_RST >= 0) {
    writecommand(0x00); // Put SPI bus in known state for TFT with CS tied low
    digitalWrite(TFT_RST, HIGH);
    delay(5);
    digitalWrite(TFT_RST, LOW);
    delay(20);
    digitalWrite(TFT_RST, HIGH);
  } else

    delay(150); // Wait for reset to complete

  begin_tft_write();

  {

    pinMode(TFT_BUSY, INPUT);
    pinMode(TFT_ENABLE, OUTPUT);
    digitalWrite(TFT_ENABLE, HIGH);
    writecommand(0x01); // POWER SETTING
    writedata(0x07);
    writedata(0x07); // VGH=20V,VGL=-20V
    writedata(0x3f); // VDH=15V
    writedata(0x3f); // VDL=-15V

    // Enhanced display drive(Add 0x06 command)
    writecommand(0x06); // Booster Soft Start
    writedata(0x17);
    writedata(0x17);
    writedata(0x28);
    writedata(0x17);

    writecommand(0x04); // POWER ON
    delay(100);

    CHECK_BUSY();

    writecommand(0X00); // PANNEL SETTING
    writedata(0x1F);    // KW-3f   KWR-2F BWROTP 0f BWOTP 1f

    writecommand(0x61); // tres
    writedata(EPD_WIDTH >> 8);
    writedata(EPD_WIDTH & 0xFF);
    writedata(EPD_HEIGHT >> 8);
    writedata(EPD_HEIGHT & 0xFF);

    writecommand(0X15);
    writedata(0x00);

    writecommand(0X50); // VCOM AND DATA INTERVAL SETTING
    writedata(0x10);
    writedata(0x07);

    writecommand(0X60); // TCON SETTING
    writedata(0x22);
  }

  end_tft_write();
}

/***************************************************************************************
** Function name:           writecommand
** Description:             Send an 8-bit command to the TFT
***************************************************************************************/
void TFT_eSPI::writecommand(uint8_t c) {
  begin_tft_write();
  DC_C;
  tft_Write_8(c);
  DC_D;
  end_tft_write();
}

/***************************************************************************************
** Function name:           writedata
** Description:             Send a 8-bit data value to the TFT
***************************************************************************************/
void TFT_eSPI::writedata(uint8_t d) {
  begin_tft_write();
  DC_D; // Play safe, but should already be in data mode
  tft_Write_8(d);
  CS_L; // Allow more hold time for low VDI rail
  end_tft_write();
}

static inline bool uc8179_wait_busy_ready(uint32_t timeout_ms = 3000) {
  uint32_t start_ms = millis();
  do {
    delay(10);
    if (digitalRead(TFT_BUSY))
      break;
    if ((millis() - start_ms) >= timeout_ms)
      return false;
  } while (true);
  return true;
}

static inline void uc8179_spi_end_for_gpio_read(void) {
#if !defined(TFT_PARALLEL_8_BIT) && !defined(RP2040_PIO_INTERFACE)
  spi.end();
#endif
}

static inline void uc8179_spi_restore_after_gpio_read(void) {
#if !defined(TFT_PARALLEL_8_BIT) && !defined(RP2040_PIO_INTERFACE)
#  if defined(TFT_MOSI) && !defined(TFT_SPI_OVERLAP) &&                        \
      !defined(ARDUINO_ARCH_RP2040) && !defined(ARDUINO_ARCH_MBED) &&          \
      !defined(EFR32MG24B220F1536IM48)
#    if defined(NRF52840_XXAA)
  spi.setPins(TFT_MISO, TFT_SCLK, TFT_MOSI);
  spi.begin();
#    else
  spi.begin(TFT_SCLK, TFT_MISO, TFT_MOSI, -1);
#    endif
#  else
  spi.begin();
#  endif
#endif
}

static inline void uc8179_gpio_write_command(uint8_t command) {
  pinMode(TFT_SCLK, OUTPUT);
  pinMode(TFT_MOSI, OUTPUT);
  pinMode(TFT_DC, OUTPUT);
  pinMode(TFT_CS, OUTPUT);

  digitalWrite(TFT_CS, HIGH);
  digitalWrite(TFT_CS, LOW);
  digitalWrite(TFT_SCLK, LOW);
  digitalWrite(TFT_DC, LOW);

  for (uint8_t bit = 0; bit < 8; bit++) {
    digitalWrite(TFT_MOSI, (command & 0x80) ? HIGH : LOW);
    digitalWrite(TFT_SCLK, HIGH);
    digitalWrite(TFT_SCLK, LOW);
    command <<= 1;
  }

  digitalWrite(TFT_CS, HIGH);
}

static inline uint8_t uc8179_gpio_read_byte(void) {
  uint8_t data = 0;

  digitalWrite(TFT_CS, LOW);
  digitalWrite(TFT_DC, HIGH);
  digitalWrite(TFT_SCLK, LOW);

  pinMode(TFT_MOSI, INPUT);
  for (uint8_t bit = 0; bit < 8; bit++) {
    data <<= 1;
    digitalWrite(TFT_SCLK, HIGH);
    if (digitalRead(TFT_MOSI) == HIGH)
      data |= 0x01;
    digitalWrite(TFT_SCLK, LOW);
  }

  pinMode(TFT_MOSI, OUTPUT);
  digitalWrite(TFT_MOSI, HIGH);
  digitalWrite(TFT_CS, HIGH);

  return data;
}

void TFT_eSPI::uc8179ReadTemperatureRaw(uint8_t *temp1, uint8_t *temp2) {
  if (temp1)
    *temp1 = 0;
  if (temp2)
    *temp2 = 0;

  uc8179_gpio_write_command(0x40);
  if (!uc8179_wait_busy_ready())
    return;

  if (temp1)
    *temp1 = uc8179_gpio_read_byte();
  if (temp2)
    *temp2 = uc8179_gpio_read_byte();
}

void TFT_eSPI::uc8179ReadOtpUserData(uint16_t read_len, uint16_t data_offset,
                                     uint8_t *buf, uint8_t buf_len) {
  if (!buf || !buf_len)
    return;

  memset(buf, 0, buf_len);
  uc8179_gpio_write_command(0xA2);

  uint8_t count = 0;
  for (uint16_t row = 0; row < read_len; row++) {
    uint8_t temp = uc8179_gpio_read_byte();
    if (row >= data_offset && count < buf_len) {
      buf[count++] = temp;
    }
  }

  delay(20);
}

void TFT_eSPI::uc8179ProbeOtpSupport(void) {
  if (_uc8179_has_checked_otp)
    return;

  const bool resume_write = !locked;
  uint8_t usrdata1[10];
  uint8_t usrdata2[10];

  _uc8179_has_checked_otp = true;
  _uc8179_use_otp_lut = false;

  uc8179_spi_end_for_gpio_read();

  digitalWrite(TFT_RST, LOW);
  delay(20);
  digitalWrite(TFT_RST, HIGH);
  delay(20);
  if (!uc8179_wait_busy_ready()) {
    uc8179_spi_restore_after_gpio_read();
    return;
  }
  uc8179ReadTemperatureRaw(&_uc8179_temp_raw_int, &_uc8179_temp_raw_frac);

  digitalWrite(TFT_RST, LOW);
  delay(20);
  digitalWrite(TFT_RST, HIGH);
  delay(20);
  if (!uc8179_wait_busy_ready()) {
    uc8179_spi_restore_after_gpio_read();
    return;
  }
  uc8179ReadOtpUserData(0x0BED, 0x0BE3, usrdata1, sizeof(usrdata1));

  digitalWrite(TFT_RST, LOW);
  delay(20);
  digitalWrite(TFT_RST, HIGH);
  delay(20);
  if (!uc8179_wait_busy_ready()) {
    uc8179_spi_restore_after_gpio_read();
    return;
  }
  uc8179ReadOtpUserData(0x17ED, 0x17E3, usrdata2, sizeof(usrdata2));

  _uc8179_use_otp_lut = (usrdata1[0] == 0x01) || (usrdata2[0] == 0x01);

  uc8179_spi_restore_after_gpio_read();
}

/***************************************************************************************
** Function name:           drawBitmap
** Description:             Draw an image stored in an array on the TFT
***************************************************************************************/
void TFT_eSPI::drawBitmap(int16_t x, int16_t y, const uint8_t *bitmap,
                          int16_t w, int16_t h, uint16_t fgcolor,
                          uint16_t bgcolor) {
  inTransaction = true;

  int32_t i, j, byteWidth = (w + 7) / 8;

  for (j = 0; j < h; j++) {
    for (i = 0; i < w; i++) {
      if (pgm_read_byte(bitmap + j * byteWidth + i / 8) & (128 >> (i & 7)))
        drawPixel(x + i, y + j, fgcolor);
      else
        drawPixel(x + i, y + j, bgcolor);
    }
  }

  inTransaction = lockTransaction;
  end_tft_write(); // Does nothing if Sprite class uses this function
}

/***************************************************************************************
** Function name:           width
** Description:             Return the pixel width of display (per current
***************************************************************************************/
int16_t TFT_eSPI::width(void) { return TFT_WIDTH; }

/***************************************************************************************
** Function name:           height
** Description:             Return the pixel height of display (per current
***************************************************************************************/
int16_t TFT_eSPI::height(void) { return TFT_HEIGHT; }

/***************************************************************************************
** Function name:           invertDisplay
** Description:             invert the display colours i = 1 invert, i = 0
* normal
***************************************************************************************/
void TFT_eSPI::invertDisplay(bool i) {
  begin_tft_write();
  // Send the command twice as otherwise it does not always work!
  writecommand(i ? TFT_INVON : TFT_INVOFF);
  writecommand(i ? TFT_INVON : TFT_INVOFF);
  end_tft_write();
}

/***************************************************************************************
** Function name:           createSprite
** Description:             Create a sprite (bitmap) of defined width and height
***************************************************************************************/
// cast returned value to (uint8_t*) for 8-bit or (uint16_t*) for 16-bit colours
void *TFT_eSPI::createSprite(int16_t w, int16_t h) {

  if (_created)
    return _img;

  if (w < 1 || h < 1)
    return nullptr;

  _img = (uint8_t *)callocSprite(w, h);
  _created = true;
  height();
}

/***************************************************************************************
** Description:             Class destructor
***************************************************************************************/
TFT_eSPI::~TFT_eSPI(void) { deleteSprite(); }

/***************************************************************************************
** Function name:           callocSprite
** Description:             Allocate a memory area for the Sprite and return
* pointer
***************************************************************************************/
void *TFT_eSPI::callocSprite(int16_t w, int16_t h) {
  // coordinates hence will run faster in normal circumstances.
  uint8_t *ptr8 = nullptr;

  if (_bpp == 4) {
    w = (w + 1) & 0xFFFE; // width needs to be multiple of 2, with an extra "off
    ptr8 = (uint8_t *)calloc(((w * h) >> 1) + 1, sizeof(uint8_t));
    Serial.println(((w * h) >> 1) + 1);
  }

  else // Must be 1 bpp
  {
    w = (w + 7) & 0xFFF8; // width should be the multiple of 8 bits to be
    ptr8 = (uint8_t *)calloc(1 * (w >> 3) * h + 1, sizeof(uint8_t));
  }

  return ptr8;
}

/***************************************************************************************
** Function name:           setColorDepth
** Description:             Set bits per pixel for colour (1, 8 or 16)
***************************************************************************************/
void *TFT_eSPI::setColorDepth(int8_t b) {
  // Do not re-create the sprite if the colour depth does not change
  if (_bpp == b)
    return _img;

  if (b > 1)
    _bpp = 4;
  else
    _bpp = 1;

  // Can't change an existing sprite's colour depth so delete and create a new
  // one
  if (_created) {
    deleteSprite();
    return createSprite(TFT_WIDTH, TFT_HEIGHT);
  }

  return nullptr;
}

/***************************************************************************************
** Function name:           deleteSprite
** Description:             Delete the sprite to free up memory (RAM)
***************************************************************************************/
void TFT_eSPI::deleteSprite(void) {

  if (_created) {
    free(_img);
    _img = nullptr;
    _created = false;
  }
}

/***************************************************************************************
** Function name:           fillSprite
** Description:             Fill the whole sprite with defined colour
***************************************************************************************/
void TFT_eSPI::fillSprite(uint32_t color) {
  if (!_created)
    return;

  // Use memset if possible as it is super fast
  if (_bpp == 4) {
    uint8_t c = ((color & 0x0F) | (((color & 0x0F) << 4) & 0xF0));
    memset(_img, c, (TFT_WIDTH * TFT_HEIGHT) >> 1);
  } else if (_bpp == 1) {
    if (color)
      memset(_img, 0xFF, (TFT_WIDTH >> 3) * TFT_HEIGHT + 1);
    else
      memset(_img, 0x00, (TFT_WIDTH >> 3) * TFT_HEIGHT + 1);
  }
}

/***************************************************************************************
** Function name:           drawPixel
** Description:             push a single pixel at an arbitrary position
***************************************************************************************/
void TFT_eSPI::drawPixel(int32_t x, int32_t y, uint32_t color) {
  if (!_created)
    return;

  // Range checking
  if ((x < 0) || (y < 0) || (x >= TFT_WIDTH) || (y >= TFT_HEIGHT))
    return;

  if (_bpp == 4) {

    uint8_t c = color & 0x0F;
    int index = (x + y * TFT_WIDTH) >> 1;
    ;
    if ((x & 0x01) == 0) {
      _img[index] = (uint8_t)((c << 4) | (_img[index] & 0x0F));
    } else {
      _img[index] = (uint8_t)(c | (_img[index] & 0xF0));
    }
  } else // 1 bpp
  {

    if (color)
      _img[(x + y * TFT_WIDTH) >> 3] |= (0x80 >> (x & 0x7));
    else
      _img[(x + y * TFT_WIDTH) >> 3] &= ~(0x80 >> (x & 0x7));
  }
}

/***************************************************************************************
** Function name:           fillRect
** Description:             draw a filled rectangle
***************************************************************************************/
void TFT_eSPI::fillRect(int32_t x, int32_t y, int32_t w, int32_t h,
                        uint32_t color) {
  if (!_created)
    return;

  // Clipping
  if ((x >= TFT_WIDTH) || (y >= TFT_HEIGHT))
    return;

  if (x < 0) {
    w += x;
    x = 0;
  }
  if (y < 0) {
    h += y - 0;
    y = 0;
  }

  if ((x + w) > TFT_WIDTH)
    w = TFT_WIDTH - x;
  if ((y + h) > TFT_HEIGHT)
    h = TFT_HEIGHT - y;

  if ((w < 1) || (h < 1))
    return;

  int32_t yp = TFT_WIDTH * y + x;

  if (_bpp == 4) {
    uint8_t c1 = (uint8_t)color & 0x0F;
    uint8_t c2 = c1 | ((c1 << 4) & 0xF0);
    if ((x & 0x01) == 0 && (w & 0x01) == 0) {
      yp = (yp >> 1);
      while (h--) {
        memset(_img + yp, c2, (w >> 1));
        yp += (TFT_WIDTH >> 1);
      }
    } else if ((x & 0x01) == 0) {

      // same as above but you have a hangover on the right.
      yp = (yp >> 1);
      while (h--) {
        if (w > 1)
          memset(_img + yp, c2, (w - 1) >> 1);
        // handle the rightmost pixel by calling drawPixel
        drawPixel(x + w - 1, y + h, c1);
        yp += (TFT_WIDTH >> 1);
      }
    } else if ((w & 0x01) == 1) {
      yp = (yp + 1) >> 1;
      while (h--) {
        drawPixel(x, y + h, color & 0x0F);
        if (w > 1)
          memset(_img + yp, c2, (w - 1) >> 1);
        // same as above but you have a hangover on the left instead
        yp += (TFT_WIDTH >> 1);
      }
    } else {
      yp = (yp + 1) >> 1;
      while (h--) {
        drawPixel(x, y + h, color & 0x0F);
        if (w > 1)
          drawPixel(x + w - 1, y + h, color & 0x0F);
        if (w > 2)
          memset(_img + yp, c2, (w - 2) >> 1);
        // maximal hacking, single pixels on left and right.
        yp += (TFT_WIDTH >> 1);
      }
    }
  } else {
    while (h--) {
      int32_t ww = w;
      int32_t xx = x;
      while (ww--)
        drawPixel(xx++, y, color);
      y++;
    }
  }
}

EPaper::EPaper() : _sleep(true) {
  setColorDepth(EPD_COLOR_DEPTH);
  createSprite(TFT_WIDTH, TFT_HEIGHT);
}

void EPaper::begin(uint8_t wake) {
  if (wake) {
    initFromSleep();
  } else {
    init();
  }
}

void EPaper::update() {
  wake();
  EPD_SET_WINDOW(0, 0, (TFT_WIDTH - 1), (TFT_HEIGHT - 1));
  if (_bpp == 1) {
    EPD_PUSH_OLD_COLORS(TFT_WIDTH, TFT_HEIGHT, _img);
    EPD_PUSH_NEW_COLORS(TFT_WIDTH, TFT_HEIGHT, _img);
    writecommand(0x12);
    delay(1);
    CHECK_BUSY();
  } else {
    EPD_PUSH_NEW_GRAY_COLORS(TFT_WIDTH, TFT_HEIGHT, _img);
    writecommand(0x12);
    delay(1);
    CHECK_BUSY();
  }
  sleep();
}

void EPaper::updataPartial(uint16_t x, uint16_t y, uint16_t w, uint16_t h) {

  int32_t bx = x;
  int32_t by = y;
  int32_t bw = w;
  int32_t bh = h;

  if (bx < 0) {
    bw += bx;
    bx = 0;
  }
  if (by < 0) {
    bh += by;
    by = 0;
  }
  if ((bx + bw) > (int32_t)TFT_WIDTH)
    bw = (int32_t)TFT_WIDTH - bx;
  if ((by + bh) > (int32_t)TFT_HEIGHT)
    bh = (int32_t)TFT_HEIGHT - by;
  if (bw < 1 || bh < 1)
    return;

  uint16_t align_px = 8;
  if (_sleep) {
    EPD_WAKEUP_PARTIAL();
    _sleep = false;
  }

  uint16_t x0 = ((uint16_t)bx) & ~(align_px - 1);
  uint16_t x1 = ((uint16_t)(bx + bw + (align_px - 1))) & ~(align_px - 1);
  uint16_t w_aligned = x1 - x0;
  uint16_t yy = (uint16_t)by;
  uint16_t hh = (uint16_t)bh;

  uint16_t stride = TFT_WIDTH >> 3;
  uint16_t win_bytes_per_row = w_aligned >> 3;

  const uint8_t *src0 = _img + (yy * stride) + (x0 >> 3);

  size_t win_size = (size_t)win_bytes_per_row * hh;
  uint8_t *winbuf = (uint8_t *)malloc(win_size);
  if (!winbuf)
    return;

  for (uint16_t row = 0; row < hh; row++) {
    memcpy(winbuf + row * win_bytes_per_row, src0 + row * stride,
           win_bytes_per_row);
  }

  EPD_SET_WINDOW(x0, yy, x0 + w_aligned - 1, yy + hh - 1);
  EPD_PUSH_NEW_COLORS(w_aligned, hh, winbuf);
  writecommand(0x12);
  delay(1);
  CHECK_BUSY();

  free(winbuf);
  sleep();
}

void EPaper::initGrayMode() {
  if (_bpp == 4) {
    return;
  }
  if (_created) {
    deleteSprite();
  }
  setColorDepth(4);
  createSprite(TFT_WIDTH, TFT_HEIGHT);
  fillSprite(TFT_WHITE);
}

void EPaper::deinitGrayMode() {
  if (_bpp == 1)
    return;

  if (_created) {
    deleteSprite();
    _bpp = 1;
  }
  setColorDepth(EPD_COLOR_DEPTH);
  createSprite(TFT_WIDTH, TFT_HEIGHT);
  fillSprite(TFT_WHITE);
}

void EPaper::sleep() {
  if (_sleep)
    return;
  writecommand(0X50);
  writedata(0xf7);
  writecommand(0x02);
  CHECK_BUSY();
  writecommand(0x07);
  writedata(0xA5);
  _sleep = true;
}

void EPaper::wake() {
  if (!_sleep)
    return;
  if (_bpp == 1)
    EPD_WAKEUP();
  else {
    EPD_WAKEUP_GRAY();
  }
  _sleep = false;
}

/// ###

void rust_println(const char *s) { Serial.println(s); }

EPaper epaper = EPaper(); // Invoke library, pins defined in User_Setup.h

/// C++ functions called from Rust
extern "C" {

void drawPixel(int x, int y, unsigned int color) {
  epaper.drawPixel(x, y, color);
}

void fillRect(int32_t x, int32_t y, int32_t w, int32_t h, uint32_t color) {
  epaper.fillRect(x, y, w, h, color);
}

void update() { epaper.update(); }

void drawBitmap(int16_t x, int16_t y, const uint8_t *bitmap, int16_t w,
                int16_t h, uint16_t color) {
  epaper.drawBitmap(x, y, bitmap, w, h, color, ~color);
}

void updataPartial(uint16_t x, uint16_t y, uint16_t w, uint16_t h) {
  epaper.updataPartial(x, y, w, h);
}

void println(const char *s) { Serial.println(s); }

void printnum(int x) { Serial.println(x); }

void ffi_delay(int ms) { delay(ms); }

void initGrayMode() { epaper.initGrayMode(); }

void deinitGrayMode() { epaper.deinitGrayMode(); }
}

// SET_LOOP_TASK_STACK_SIZE(12 * 1024);
SET_LOOP_TASK_STACK_SIZE(64 * 1024);

void setup(void) {
  // Initialize serial communication
  Serial.begin(115200);
  epaper.begin();
  Serial.println("meow");
  rust_setup();
}

int fc = 0;

void loop() {
  rust_loop(fc);
  fc++;
}
