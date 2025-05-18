use std::collections::HashMap;

pub enum ChartType {
    Dots,
    Lines,
}

#[derive(Clone)]
pub struct Screen {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
}

impl Screen {
    pub fn new(width: usize, height: usize) -> Self {
        let buffer = vec![0x000000; width * height]; // Initialize with black color
        Screen {
            width,
            height,
            buffer,
        }
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0x000000); // Fill the buffer with black color
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x] = color;
        }
    }

    pub fn draw_circle(&mut self, center_x: usize, center_y: usize, radius: usize, color: u32) {
        let y_range = center_y.saturating_sub(radius)..=center_y.saturating_add(radius);
        let x_range = center_x.saturating_sub(radius)..=center_x.saturating_add(radius);
        for y in y_range {
            for x in x_range.clone() {
                let dx = x as isize - center_x as isize;
                let dy = y as isize - center_y as isize;
                if dx * dx + dy * dy <= (radius * radius) as isize {
                    self.set_pixel(x, y, color);
                }
            }
        }
    }

    pub fn draw_bitmap(
        &mut self,
        x: usize,
        y: usize,
        bitmap: &[u8],
        width: usize,
        height: usize,
        scale: usize,
    ) {
        for row in 0..height {
            for col in 0..width {
                let px = x + col * scale;
                let py = y + row * scale;
                let color = bitmap[row * width + col] as u32;
                let color: u32 = (color << 16) | (color << 8) | color;
                self.draw_block(px, py, scale, color);
            }
        }
    }

    pub fn draw_line(
        &mut self,
        start: (usize, usize),
        end: (usize, usize),
        thickness: usize,
        color: u32,
    ) {
        let (x0, y0) = start;
        let (x1, y1) = end;

        let dx = (x1 as isize - x0 as isize).abs();
        let dy = (y1 as isize - y0 as isize).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x0;
        let mut y = y0;

        loop {
            if thickness == 1 {
                self.set_pixel(x, y, color); // Draw a single pixel
            } else {
                self.draw_circle(x, y, thickness / 2, color); // Draw a circle for thickness
            }
            if x == x1 && y == y1 {
                break;
            }
            let err2 = err * 2;
            if err2 > -dy {
                err -= dy;
                x = (x as isize + sx) as usize;
            }
            if err2 < dx {
                err += dx;
                y = (y as isize + sy) as usize;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_chart(
        &mut self,
        chart_type: ChartType,
        has_grid: bool,
        data: &[(f32, f32)],
        x_range: (f32, f32),
        y_range: (f32, f32),
        thickness: usize,
        color: u32,
    ) {
        let (x_min, x_max) = x_range;
        let (y_min, y_max) = y_range;

        // Conversione da coordinate logiche a coordinate dello schermo
        let screen_c = self.clone();
        let to_screen = |x: f32, y: f32| -> (usize, usize) {
            let x_norm = (x - x_min) / (x_max - x_min);
            let y_norm = (y - y_min) / (y_max - y_min);
            let sx =
                (x_norm * screen_c.width as f32).clamp(0.0, (screen_c.width - 1) as f32) as usize;
            let sy = ((1.0 - y_norm) * screen_c.height as f32)
                .clamp(0.0, (screen_c.height - 1) as f32) as usize;
            (sx, sy)
        };

        // Disegna assi
        let zero_x = if x_min <= 0.0 && x_max >= 0.0 {
            Some(to_screen(0.0, y_min).0)
        } else {
            None
        };

        let zero_y = if y_min <= 0.0 && y_max >= 0.0 {
            Some(to_screen(x_min, 0.0).1)
        } else {
            None
        };

        if let Some(x) = zero_x {
            for y in 0..self.height {
                self.set_pixel(x, y, 0x444444); // asse y
            }
        }

        if let Some(y) = zero_y {
            for x in 0..self.width {
                self.set_pixel(x, y, 0x444444); // asse x
            }
        }

        // Disegna i punti
        match chart_type {
            ChartType::Dots => {
                for &(x, y) in data {
                    let (sx, sy) = to_screen(x, y);
                    self.draw_circle(sx, sy, thickness, color);
                }
            }
            ChartType::Lines => {
                for window in data.windows(2) {
                    if let [a, b] = window {
                        let (x0, y0) = to_screen(a.0, a.1);
                        let (x1, y1) = to_screen(b.0, b.1);
                        self.draw_line((x0, y0), (x1, y1), thickness, color);
                    }
                }
            }
        }

        // Opzionale: grid non implementata ora
        if has_grid {
            // TODO: disegno griglia (ticks, linee secondarie)
        }
    }

    pub fn draw_text(&mut self, x: usize, y: usize, text: &str, color: u32, scale: usize) {
        let font = get_font_5x7();
        let char_width = 5;
        let _char_height = 7;
        let spacing = 1;

        for (i, c) in text.chars().enumerate() {
            if let Some(bitmap) = font.get(&c) {
                let x_offset = x + i * (char_width + spacing) * scale;

                for (row, row_bits) in bitmap.iter().enumerate() {
                    for col in 0..char_width {
                        if (row_bits >> (char_width - 1 - col)) & 1 == 1 {
                            let px = x_offset + col * scale;
                            let py = y + row * scale;
                            self.draw_block(px, py, scale, color);
                        }
                    }
                }
            }
        }
    }

    fn draw_block(&mut self, x: usize, y: usize, size: usize, color: u32) {
        for dy in 0..size {
            for dx in 0..size {
                self.set_pixel(x + dx, y + dy, color);
            }
        }
    }
}

// Sample font with just a few letters
fn get_font_5x7() -> HashMap<char, [u8; 7]> {
    use std::iter::FromIterator;
    HashMap::from_iter([
        (
            'A',
            [
                0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
            ],
        ),
        (
            'B',
            [
                0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
            ],
        ),
        (
            'C',
            [
                0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
            ],
        ),
        (
            'D',
            [
                0b11100, 0b10010, 0b10001, 0b10001, 0b10001, 0b10010, 0b11100,
            ],
        ),
        (
            'E',
            [
                0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
            ],
        ),
        (
            'F',
            [
                0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
            ],
        ),
        (
            'G',
            [
                0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
            ],
        ),
        (
            'H',
            [
                0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
            ],
        ),
        (
            'I',
            [
                0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
            ],
        ),
        (
            'J',
            [
                0b00001, 0b00001, 0b00001, 0b00001, 0b10001, 0b10001, 0b01110,
            ],
        ),
        (
            'K',
            [
                0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
            ],
        ),
        (
            'L',
            [
                0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
            ],
        ),
        (
            'M',
            [
                0b10001, 0b11011, 0b10101, 0b10001, 0b10001, 0b10001, 0b10001,
            ],
        ),
        (
            'N',
            [
                0b10001, 0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001,
            ],
        ),
        (
            'O',
            [
                0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ],
        ),
        (
            'P',
            [
                0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
            ],
        ),
        (
            'Q',
            [
                0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
            ],
        ),
        (
            'R',
            [
                0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
            ],
        ),
        (
            'S',
            [
                0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
            ],
        ),
        (
            'T',
            [
                0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
            ],
        ),
        (
            'U',
            [
                0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ],
        ),
        (
            'V',
            [
                0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
            ],
        ),
        (
            'W',
            [
                0b10001, 0b10001, 0b10001, 0b10001, 0b10101, 0b11011, 0b10001,
            ],
        ),
        (
            'X',
            [
                0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
            ],
        ),
        (
            'Y',
            [
                0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
            ],
        ),
        (
            'Z',
            [
                0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
            ],
        ),
        (
            ' ',
            [
                0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
            ],
        ),
        (
            '0',
            [
                0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
            ],
        ),
        (
            '1',
            [
                0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
            ],
        ),
        (
            '2',
            [
                0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
            ],
        ),
        (
            '3',
            [
                0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110,
            ],
        ),
        (
            '4',
            [
                0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
            ],
        ),
        (
            '5',
            [
                0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
            ],
        ),
        (
            '6',
            [
                0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
            ],
        ),
        (
            '7',
            [
                0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
            ],
        ),
        (
            '8',
            [
                0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
            ],
        ),
        (
            '9',
            [
                0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
            ],
        ),
        (
            '!',
            [
                0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100, 0b00000,
            ],
        ),
        (
            '"',
            [
                0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
            ],
        ),
        (
            '#',
            [
                0b01010, 0b11111, 0b01010, 0b01010, 0b11111, 0b01010, 0b00000,
            ],
        ),
        (
            '$',
            [
                0b00100, 0b01111, 0b10100, 0b01110, 0b00101, 0b11110, 0b00100,
            ],
        ),
        (
            '%',
            [
                0b11000, 0b11001, 0b00010, 0b00100, 0b01000, 0b10011, 0b00011,
            ],
        ),
        (
            '&',
            [
                0b01100, 0b10010, 0b10100, 0b01000, 0b10101, 0b10010, 0b01101,
            ],
        ),
        (
            '\'',
            [
                0b00100, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
            ],
        ),
        (
            '(',
            [
                0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
            ],
        ),
        (
            ')',
            [
                0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
            ],
        ),
        (
            '*',
            [
                0b00000, 0b00100, 0b10101, 0b01110, 0b10101, 0b00100, 0b00000,
            ],
        ),
        (
            '+',
            [
                0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
            ],
        ),
        (
            ',',
            [
                0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000,
            ],
        ),
        (
            '-',
            [
                0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
            ],
        ),
        (
            '.',
            [
                0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00110, 0b00110,
            ],
        ),
        (
            '/',
            [
                0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000,
            ],
        ),
        (
            ':',
            [
                0b00000, 0b00110, 0b00110, 0b00000, 0b00110, 0b00110, 0b00000,
            ],
        ),
        (
            ';',
            [
                0b00000, 0b00110, 0b00110, 0b00000, 0b00110, 0b00100, 0b01000,
            ],
        ),
        (
            '<',
            [
                0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010,
            ],
        ),
        (
            '=',
            [
                0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
            ],
        ),
        (
            '>',
            [
                0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000,
            ],
        ),
        (
            '?',
            [
                0b01110, 0b10001, 0b00010, 0b00100, 0b00100, 0b00000, 0b00100,
            ],
        ),
        (
            '@',
            [
                0b01110, 0b10001, 0b10111, 0b10101, 0b10111, 0b10000, 0b01110,
            ],
        ),
        (
            '[',
            [
                0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110,
            ],
        ),
        (
            '\\',
            [
                0b10000, 0b01000, 0b00100, 0b00010, 0b00001, 0b00000, 0b00000,
            ],
        ),
        (
            ']',
            [
                0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110,
            ],
        ),
        (
            '^',
            [
                0b00100, 0b01010, 0b10001, 0b00000, 0b00000, 0b00000, 0b00000,
            ],
        ),
        (
            '_',
            [
                0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
            ],
        ),
        (
            '`',
            [
                0b00100, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
            ],
        ),
    ])
}
