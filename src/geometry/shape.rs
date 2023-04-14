use std::fmt::{Debug, Error, Formatter};
use std::hash::{Hash, Hasher};

use nalgebra::Vector2;
use num::Float;

use crate::global::RADIUS;

#[derive(Copy, Clone)]
pub struct Square(pub Vector2<f64>, pub Vector2<f64>);

impl Debug for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", [((self.0).x, (self.0).y), ((self.1).x, (self.1).y)])
    }
}

#[derive(PartialEq, Copy, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub mass: f64,
}

impl Debug for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", (self.x, self.y))
    }
}

impl Eq for Point {}

impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.x.integer_decode(), self.y.integer_decode(), self.mass.integer_decode()).hash(state);
    }
}

impl Point {
    pub fn coords(&self) -> Vector2<f64> {
        Vector2::new(self.x, self.y)
    }
    pub fn update_by(&mut self, data: &Vector2<f64>) {
        self.x += data.x;
        self.y += data.y;
    }
}

impl Square {
    pub(crate) fn contains(&self, x: &Point) -> bool {
        self.0.x > x.x + RADIUS
            && self.0.y > x.y + RADIUS
            && self.1.x < x.x - RADIUS
            && self.1.y < x.y - RADIUS
    }
    pub fn touch(&self, x: &Point) -> bool {
        static DIST: f64 = RADIUS * RADIUS;
        (self.0.x - x.x) * (self.0.x - x.x) <= DIST
            || (self.1.x - x.x) * (self.1.x - x.x) <= DIST
            || (self.0.y - x.y) * (self.0.y - x.y) <= DIST
            || (self.1.y - x.y) * (self.1.y - x.y) <= DIST
    }
    pub fn can_touch(&self, x: &Point) -> bool {
        static DIST: f64 = 9.0 * RADIUS * RADIUS;
        let mid = (self.0 + self.1) / 2.0;
        (mid.x - x.x) * (mid.x - x.x) <= DIST
            || (mid.y - x.y) * (mid.y - x.y) <= DIST
    }
}

pub fn check(p: &Point, q: &Point) -> bool {
    let a = p.x - q.x;
    let b = p.y - q.y;
    a * a + b * b < 4.0 * RADIUS * RADIUS
}