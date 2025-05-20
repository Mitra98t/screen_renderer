use nalgebra::Vector2;

use crate::{Pixel, Stroke, StrokeType};

pub trait Shape {
    fn is_in_shape(&self, pos: Vector2<usize>) -> Option<Pixel>;
    fn set_pos(&mut self, pos: Vector2<usize>);
}

#[derive(Debug, Clone)]
pub struct Circle {
    pub center: Vector2<usize>,
    pub radius: usize,
    pub fill: Option<Pixel>,
    pub stroke: Stroke,
}

impl Shape for Circle {
    fn is_in_shape(&self, pos: Vector2<usize>) -> Option<Pixel> {
        let (inner_rad, outer_rad) = match self.stroke.stroke_type {
            StrokeType::Inner => (
                if self.radius < self.stroke.width {
                    0
                } else {
                    self.radius - self.stroke.width
                },
                self.radius,
            ),
            StrokeType::Outer => (self.radius, self.radius + self.stroke.width),
            StrokeType::Center => (
                if self.radius < self.stroke.width / 2 {
                    0
                } else {
                    self.radius - self.stroke.width / 2
                },
                self.radius + self.stroke.width / 2,
            ),
        };
        let inner_sq = inner_rad * inner_rad;
        let outer_sq = outer_rad * outer_rad;

        let (inner_sq, outer_sq) = (inner_sq as isize, outer_sq as isize);

        // distance squared from center to pos
        let dist_sq = (self.center.x as isize - pos.x as isize).pow(2)
            + (self.center.y as isize - pos.y as isize).pow(2);
        if dist_sq < inner_sq {
            return self.fill;
        }
        if dist_sq < outer_sq {
            return Some(self.stroke.color);
        }

        None
    }
    fn set_pos(&mut self, center: Vector2<usize>) {
        self.center = center;
    }
}

impl Circle {
    pub fn new(center: Vector2<usize>, radius: usize, fill: Option<Pixel>, stroke: Stroke) -> Self {
        Circle {
            center,
            radius,
            fill,
            stroke,
        }
    }
}
