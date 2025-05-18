use crate::{Pixel, Stroke};

pub enum ChartType {
    Dots,
    Lines,
}

pub struct Chart {
    pub width: usize,
    pub height: usize,
    x_range: (f32, f32),
    y_range: (f32, f32),
    fill: Pixel,
    stroke: Stroke,
    chart_axis: Stroke,
    pub buffer: Vec<Pixel>,
}

impl Chart {
    pub fn new(width: usize, height: usize, x_range: (f32, f32), y_range: (f32, f32)) -> Self {
        let fill = 0x000000; // Default fill color (black)
        let stroke = Stroke {
            color: 0xFFFFFF,                       // Default stroke color (white)
            width: 1,                              // Default stroke width
            stroke_type: crate::StrokeType::Outer, // Default stroke type (solid)
        };

        Chart {
            width,
            height,
            x_range,
            y_range,
            fill,
            stroke,
            chart_axis: Stroke {
                color: 0x444444, // Default axis color (white)
                width: 1,        // Default axis width
                stroke_type: crate::StrokeType::Outer,
            },
            buffer: vec![0x000000; width * height], // Initialize with black color
        }
    }

    pub fn fill(&mut self, color: u32) {
        self.fill = color;
    }

    pub fn stroke(&mut self, color: u32, width: usize) {
        self.stroke.color = color;
        self.stroke.width = width;
    }

    pub fn stroke_color(&mut self, color: Pixel) {
        self.stroke.color = color;
    }

    pub fn stroke_width(&mut self, width: usize) {
        self.stroke.width = width;
    }

    pub fn chart_color(&mut self, color: Pixel) {
        self.chart_axis.color = color;
    }
    pub fn chart_width(&mut self, width: usize) {
        self.chart_axis.width = width;
    }

    fn is_in_bounds(&mut self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: Pixel) {
        if self.is_in_bounds(x, y) {
            self.buffer[y * self.width + x] = color;
        }
    }

    fn circle(&mut self, cx: usize, cy: usize, radius: usize) {
        for y in -(radius as isize)..=radius as isize {
            for x in -(radius as isize)..=(radius as isize) {
                let distance_sq = x * x + y * y;
                if distance_sq <= (radius as isize) * (radius as isize) {
                    let px = cx as isize + x;
                    let py = cy as isize + y;
                    if self.is_in_bounds(px as usize, py as usize) {
                        self.set_pixel(px as usize, py as usize, self.stroke.color);
                    }
                }
            }
        }
    }
    fn line(&mut self, start: (usize, usize), end: (usize, usize)) {
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
            if self.stroke.width == 1 {
                self.set_pixel(x, y, self.stroke.color); // Draw a single pixel
            } else {
                self.circle(x, y, self.stroke.width / 2); // Draw a circle for thickness
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

    fn to_buffer_coordinates(&self, x: f32, y: f32) -> (usize, usize) {
        let (x_min, x_max) = self.x_range;
        let (y_min, y_max) = self.y_range;

        let x_norm = (x - x_min) / (x_max - x_min);
        let y_norm = (y - y_min) / (y_max - y_min);

        let bx = (x_norm * self.width as f32).clamp(0.0, (self.width - 1) as f32) as usize;
        let by =
            ((1.0 - y_norm) * self.height as f32).clamp(0.0, (self.height - 1) as f32) as usize;

        (bx, by)
    }

    // render to buffer
    fn chart_axis(&mut self) {
        let (x_min, x_max) = self.x_range;
        let (y_min, y_max) = self.y_range;

        // Draw x-axis
        let zero_x = if x_min <= 0.0 && x_max >= 0.0 {
            Some(self.to_buffer_coordinates(0., y_min).0)
        } else {
            None
        };
        if let Some(x) = zero_x {
            let stroke = self.stroke;
            self.stroke = self.chart_axis;
            self.line((x, 0), (x, self.height - 1)); // asse x
            self.stroke = stroke;
        }

        // Draw y-axis
        let zero_y = if y_min <= 0.0 && y_max >= 0.0 {
            Some(self.to_buffer_coordinates(x_min, 0.).1)
        } else {
            None
        };
        if let Some(y) = zero_y {
            let stroke = self.stroke;
            self.stroke = self.chart_axis;
            self.line((0, y), (self.width - 1, y)); // asse x
            self.stroke = stroke;
        }
    }

    pub fn draw(&mut self, chart_type: ChartType, data: &[(f32, f32)]) {
        self.chart_axis();
        // Disegna i punti
        match chart_type {
            ChartType::Dots => {
                for &(x, y) in data {
                    let (bx, by) = self.to_buffer_coordinates(x, y);
                    self.circle(bx, by, self.stroke.width / 2);
                }
            }
            ChartType::Lines => {
                for i in 0..data.len() - 1 {
                    let (x1, y1) = data[i];
                    let (x2, y2) = data[i + 1];

                    let (bx1, by1) = self.to_buffer_coordinates(x1, y1);
                    let (bx2, by2) = self.to_buffer_coordinates(x2, y2);

                    self.line((bx1, by1), (bx2, by2));
                }
            }
        }
    }
}
