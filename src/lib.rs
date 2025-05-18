use ::nalgebra::Vector2;
use font::get_font_5x7;
use minifb::{Window, WindowOptions};

pub mod minifb {
    pub use minifb::*;
}
pub mod nalgebra {
    pub use nalgebra::*;
}

pub mod chart;
pub mod font;

pub type Pixel = u32;

#[derive(Clone, Copy)]
pub enum StrokeType {
    Inner,
    Outer,
    Center,
}

#[derive(Clone, Copy)]
pub struct Stroke {
    pub color: Pixel,
    pub width: usize,
    pub stroke_type: StrokeType,
}

pub struct Screen {
    width: usize,
    height: usize,
    pub buffer: Vec<Pixel>,
    pub window: Window,
    stroke: Stroke,
    fill: Pixel,
}

impl Screen {
    pub fn new(width: usize, height: usize, window_name: &str, opts: WindowOptions) -> Self {
        let buffer = vec![0x000000; width * height]; // Initialize with black color

        let mut window = Window::new(window_name, width, height, opts).unwrap_or_else(|e| {
            panic!("Window creation failed: {}", e);
        });
        window.set_target_fps(30);

        let stroke = Stroke {
            color: 0xFFFFFF, // Default stroke color (white)
            width: 1,        // Default stroke width
            stroke_type: StrokeType::Center,
        };

        let fill = 0x000000; // Default fill color (black)

        Screen {
            width,
            height,
            buffer,
            window,
            stroke,
            fill,
        }
    }

    pub fn stroke_color(&mut self, color: Pixel) {
        self.stroke.color = color;
    }
    pub fn stroke_width(&mut self, width: usize) {
        self.stroke.width = width;
    }
    pub fn stroke_type(&mut self, s_type: StrokeType) {
        self.stroke.stroke_type = s_type;
    }

    pub fn fill(&mut self, color: Pixel) {
        self.fill = color;
    }

    pub fn target_fps(&mut self, fps: usize) {
        self.window.set_target_fps(fps);
    }

    // FIX: check srtoke width
    pub fn circle(&mut self, center: Vector2<usize>, radius: usize) {
        let (inner_rad, outer_rad) = match self.stroke.stroke_type {
            StrokeType::Inner => (
                if radius < self.stroke.width {
                    0
                } else {
                    radius - self.stroke.width
                },
                radius,
            ),
            StrokeType::Outer => (radius, radius + self.stroke.width),
            StrokeType::Center => (
                if radius < self.stroke.width / 2 {
                    0
                } else {
                    radius - self.stroke.width / 2
                },
                radius + self.stroke.width / 2,
            ),
        };
        let (inner_rad, outer_rad) = (inner_rad as isize, outer_rad as isize);
        let inner_sq = inner_rad * inner_rad;
        let outer_sq = outer_rad * outer_rad;

        for y in -outer_rad..=outer_rad {
            for x in -outer_rad..=outer_rad {
                let point = Vector2::new(x, y);
                let dist_sq = point.dot(&point);
                if dist_sq <= outer_sq {
                    let canvas_point = Vector2::new(center.x as isize + x, center.y as isize + y);
                    let canvas_point = canvas_point.map(|v| v as usize);
                    if dist_sq > inner_sq {
                        self.set_pixel(canvas_point, self.stroke.color); // Draw the stroke
                    } else {
                        self.set_pixel(canvas_point, self.fill); // Fill the circle
                    }
                }
            }
        }
    }
    pub fn line(&mut self, start: Vector2<usize>, end: Vector2<usize>) {
        // Bresenham's line algorithm
        let (x0, y0) = (start.x, start.y);
        let (x1, y1) = (end.x, end.y);

        let dx = (x1 as isize - x0 as isize).abs();
        let dy = (y1 as isize - y0 as isize).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x0;
        let mut y = y0;

        loop {
            let position = Vector2::new(x, y);
            if self.stroke.width == 1 {
                self.set_pixel(position, self.stroke.color); // Draw a single pixel
            } else {
                let fill_color = self.fill;
                self.fill(self.stroke.color); // Fill the stroke color
                self.circle(position, self.stroke.width / 2); // Draw a circle for thickness
                self.fill(fill_color); // Restore the fill color
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

    pub fn solid(&mut self, color: Pixel) {
        self.buffer.fill(color);
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0);
    }

    pub fn is_in_bounds(&self, pos: Vector2<usize>) -> bool {
        pos.x < self.width && pos.y < self.height
    }

    pub fn set_pixel(&mut self, pos: Vector2<usize>, pixel: Pixel) {
        if self.is_in_bounds(pos) {
            self.buffer[pos.y * self.width + pos.x] = pixel;
        }
    }

    pub fn text(&mut self, pos: Vector2<usize>, text: &str, scale: usize) {
        let (x, y) = (pos.x, pos.y);
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
                            let fill_color = self.fill;
                            let stroke_width = self.stroke.width;
                            self.fill(self.stroke.color); // Fill the stroke color
                            self.stroke_width(0);
                            self.rect(Vector2::new(px, py), scale, scale, false);
                            self.fill(fill_color); // Restore the fill color
                            self.stroke_width(stroke_width);
                        }
                    }
                }
            }
        }
    }

    pub fn rect(&mut self, pos: Vector2<usize>, width: usize, height: usize, only_stroke: bool) {
        let mut pos = pos;
        let (outer_width, outer_height, inner_width, outer_heigth) = match self.stroke.stroke_type {
            StrokeType::Inner => (
                width,
                height,
                if width < self.stroke.width * 2 {
                    0
                } else {
                    width - self.stroke.width * 2
                },
                if height < self.stroke.width * 2 {
                    0
                } else {
                    height - self.stroke.width * 2
                },
            ),
            StrokeType::Outer => {
                pos.x -= self.stroke.width;
                pos.y -= self.stroke.width;
                (
                    width + self.stroke.width * 2,
                    height + self.stroke.width * 2,
                    width,
                    height,
                )
            }
            StrokeType::Center => {
                pos.x -= self.stroke.width / 2;
                pos.y -= self.stroke.width / 2;
                (
                    width + self.stroke.width,
                    height + self.stroke.width,
                    if width < self.stroke.width {
                        0
                    } else {
                        width - self.stroke.width
                    },
                    if height < self.stroke.width {
                        0
                    } else {
                        height - self.stroke.width
                    },
                )
            }
        };
        for dy in 0..outer_height {
            for dx in 0..outer_width {
                let position = Vector2::new(pos.x + dx, pos.y + dy);
                match self.stroke.stroke_type {
                    StrokeType::Inner => {
                        if dx >= self.stroke.width
                            && dx < outer_width - self.stroke.width
                            && dy >= self.stroke.width
                            && dy < outer_height - self.stroke.width
                        {
                            if !only_stroke {
                                self.set_pixel(position, self.fill);
                            } // Fill the inner rectangle
                        } else {
                            self.set_pixel(position, self.stroke.color); // Draw the stroke
                        }
                    }
                    StrokeType::Outer => {
                        if dx < self.stroke.width
                            || dx >= outer_width - self.stroke.width
                            || dy < self.stroke.width
                            || dy >= outer_height - self.stroke.width
                        {
                            self.set_pixel(position, self.stroke.color); // Draw the stroke
                        } else if !only_stroke {
                            self.set_pixel(position, self.fill);
                        }
                    }
                    StrokeType::Center => {
                        if dx < self.stroke.width
                            || dx >= outer_width - self.stroke.width
                            || dy < self.stroke.width
                            || dy >= outer_height - self.stroke.width
                        {
                            self.set_pixel(position, self.stroke.color); // Draw the stroke
                        } else if !only_stroke {
                            self.set_pixel(position, self.fill);
                        }
                    }
                }
            }
        }
    }

    pub fn draw_buffer(
        &mut self,
        pos: Vector2<usize>,
        buffer: &[Pixel],
        width: usize,
        height: usize,
    ) {
        let (x0, y0) = (pos.x, pos.y);
        // draw buffer
        for y in 0..height {
            for x in 0..width {
                let pixel = buffer[y * width + x];
                self.set_pixel(Vector2::new(x + x0, y + y0), pixel);
            }
        }
        // draw outline
        let fill = self.fill;
        self.fill(self.stroke.color);
        self.rect(Vector2::new(x0, y0), width, height, true);
        self.fill(fill);
    }

    pub fn draw(&mut self) {
        self.window
            .update_with_buffer(&self.buffer, self.width, self.height)
            .unwrap();
    }

    pub fn is_window_open(&self) -> bool {
        self.window.is_open()
    }

    pub fn is_key_down(&self, key: minifb::Key) -> bool {
        self.window.is_key_down(key)
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn circles() {}
}
