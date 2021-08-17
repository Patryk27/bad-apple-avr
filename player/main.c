#define F_CPU 2000000UL

#include <avr/interrupt.h>
#include <avr/io.h>
#include <avr/pgmspace.h>
#include <util/delay.h>
#include "/tmp/video.c"

#define LCD_SCE PC1
#define LCD_RST PC2
#define LCD_DC PC3
#define LCD_DIN PC4
#define LCD_CLK PC5

/* ----- */

__attribute__((noreturn))
void panic() {
  DDRD |= _BV(PD7);

  while (1) {
    PORTD ^= _BV(PD7);
    _delay_ms(500);
  }
}

/* ----- */

uint8_t lcd_fb[504] = { 0 };

void lcd_write(uint8_t val, uint8_t as_data) {
  PORTC &= ~_BV(LCD_SCE);

  if (as_data) {
    PORTC |= _BV(LCD_DC);
  } else {
    PORTC &= ~_BV(LCD_DC);
  }

  for (uint8_t i = 0; i < 8; i += 1) {
    if ((val >> (7-i)) & 0x01) {
      PORTC |= _BV(LCD_DIN);
    } else {
      PORTC &= ~_BV(LCD_DIN);
    }

    PORTC |= _BV(LCD_CLK);
    PORTC &= ~_BV(LCD_CLK);
  }

  PORTC |= _BV(LCD_SCE);
}

void lcd_write_cmd(uint8_t val) {
  lcd_write(val, 0);
}

void lcd_write_data(uint8_t val) {
  lcd_write(val, 1);
}

void lcd_render() {
  lcd_write_cmd(0x80);
  lcd_write_cmd(0x40);

  for (int i = 0; i < 504; i += 1) {
    lcd_write_data(lcd_fb[i]);
  }
}

void lcd_init() {
  DDRC |= _BV(LCD_SCE);
  DDRC |= _BV(LCD_RST);
  DDRC |= _BV(LCD_DC);
  DDRC |= _BV(LCD_DIN);
  DDRC |= _BV(LCD_CLK);

  PORTC |= _BV(LCD_RST);
  PORTC |= _BV(LCD_SCE);
  _delay_ms(10);
  PORTC &= ~_BV(LCD_RST);
  _delay_ms(70);
  PORTC |= _BV(LCD_RST);

  PORTC &= ~_BV(LCD_SCE);
  lcd_write_cmd(0x20 | 0x01);
  lcd_write_cmd(0x13);
  lcd_write_cmd(0x06);
  lcd_write_cmd(0x80 | 0x40);
  lcd_write_cmd(0x20);
  lcd_write_cmd(0x09);

  lcd_render();

  lcd_write_cmd(0x08);
  lcd_write_cmd(0x0C);
}

void lcd_set(uint16_t x, uint16_t y, uint8_t pixel) {
  uint8_t *p = &lcd_fb[x + y / 8 * 84];

  if (pixel) {
    *p &= ~_BV(y % 8);
  } else {
    *p |= _BV(y % 8);
  }
}

uint8_t lcd_get(uint16_t x, uint16_t y) {
  return lcd_fb[x + y / 8 * 84] & _BV(y % 8);
}

/* ----- */

#define VIDEO_WIDTH 84
#define VIDEO_HEIGHT 48
#define VIDEO_BLOCK_WIDTH 4
#define VIDEO_BLOCK_HEIGHT 2
#define VIDEO_XBLOCKS (VIDEO_WIDTH / VIDEO_BLOCK_WIDTH)
#define VIDEO_YBLOCKS (VIDEO_HEIGHT / VIDEO_BLOCK_HEIGHT)

uint16_t video_pos = (uint16_t) video;
uint8_t video_packet[512] = { 0 };
uint16_t video_packet_len = 0;

uint8_t video_peek_u8() {
  return pgm_read_byte(video_pos);
}

uint8_t video_read_u8() {
  uint8_t val = pgm_read_byte(video_pos);
  video_pos += 1;
  
  return val;
}

uint16_t video_read_u16() {
  uint16_t val = pgm_read_word(video_pos);
  video_pos += 2;
  
  return val;
}

void video_process_packet_iframe() {
  uint16_t idx = 0;

  for (uint8_t bx = 0; bx < VIDEO_XBLOCKS; bx += 1) {
    for (uint8_t by = 0; by < VIDEO_YBLOCKS; by += 1) {
      uint8_t x0 = bx * VIDEO_BLOCK_WIDTH;
      uint8_t x1 = x0 + VIDEO_BLOCK_WIDTH;

      uint8_t y0 = by * VIDEO_BLOCK_HEIGHT;
      uint8_t y1 = y0 + VIDEO_BLOCK_HEIGHT;

      for (uint8_t x = x0; x < x1; x += 1) {
        for (uint8_t y = y0; y < y1; y += 1) {
          uint8_t pixel = video_packet[idx / 8] & _BV(idx % 8);
          idx += 1;

          lcd_set(VIDEO_WIDTH - x, VIDEO_HEIGHT - y, pixel);
        }
      }
    }
  }
}

void video_process_packet_dframe() {
  //
}

void video_process_packet_pframe() {
  uint16_t idx = 0;

  for (uint8_t bx = 0; bx < VIDEO_XBLOCKS; bx += 1) {
    for (uint8_t by = 0; by < VIDEO_YBLOCKS; by += 1) {
      uint8_t x0 = bx * VIDEO_BLOCK_WIDTH;
      uint8_t x1 = x0 + VIDEO_BLOCK_WIDTH;

      uint8_t y0 = by * VIDEO_BLOCK_HEIGHT;
      uint8_t y1 = y0 + VIDEO_BLOCK_HEIGHT;

      for (uint8_t x = x0; x < x1; x += 1) {
        for (uint8_t y = y0; y < y1; y += 1) {
          uint8_t toggle = video_packet[idx / 8] & _BV(idx % 8);
          idx += 1;

          if (toggle) {
            /* TODO */
            /* uint8_t pixel = lcd_get(VIDEO_WIDTH - x, VIDEO_HEIGHT - y); */
            /* lcd_set(VIDEO_WIDTH - x, VIDEO_HEIGHT - y, !pixel); */
          }
        }
      }
    }
  }
}

uint8_t video_process_packet() {
  // Read packet type
  uint8_t ty = video_read_u8();

  // Read packet length
  video_packet_len = video_read_u16();

  // Read packet body
  uint16_t pos = 0;

  while (pos < video_packet_len) {
    uint8_t b1 = video_read_u8();
    uint8_t b2 = video_peek_u8();

    if (b1 == b2) {
      video_read_u8();
      uint8_t len = video_read_u8();

      while (len--) {
        video_packet[pos++] = b1;
      }
    } else {
      video_packet[pos++] = b1;
    }
  }

  // Dispatch
  switch (ty) {
    case 0:
      video_process_packet_iframe();
      return 1;

    case 1:
      video_process_packet_dframe();
      return 1;

    case 2:
      video_process_packet_pframe();
      return 1;

    case 3:
      return 0;

    default:
      panic();
  }
}

/* ----- */

int main(void) {
  lcd_init();

  while (video_process_packet()) {
    lcd_render();
  }

  while (1) {
    //
  }
}
