use std::f64::EPSILON;

use crate::geometry::SimpleBody;
use crate::global::*;

pub fn handle_impact(i: &SimpleBody, j: &SimpleBody, res: &mut (f64, f64, f64, f64)) {
    let delta_x = i.x - j.x;
    let delta_y = i.y - j.y;
    let dist = delta_x * delta_x + delta_y * delta_y;
    if dist < EPSILON {
        return;
    } else if dist <= RADIUS * RADIUS * 4.0 {
        let dot = delta_x * (i.vx - j.vx)
            + delta_y * (i.vy - j.vy);
        let scale = 2.0 / (i.m + j.m) * dot / dist;
        res.0 -= scale * delta_x * j.m;
        res.1 -= scale * delta_y * j.m;
    } else {
        let scale = G / dist / dist.sqrt();
        res.2 -= delta_x * scale * j.m;
        res.3 -= delta_y * scale * j.m;
    }
}

pub fn update(i: &mut SimpleBody) {
    let rw: f64 = *WIDTH / *SCALE_FACTOR;
    let rh: f64 = *HEIGHT / *SCALE_FACTOR;
    if i.vx.is_nan() {
        i.vx = 0.0;
        i.x = 0.618 * *WIDTH / *SCALE_FACTOR;
    }
    if i.vy.is_nan() {
        i.vy = 0.0;
        i.y = 0.618 * *HEIGHT / *SCALE_FACTOR;
    }
    i.x += i.vx * ALPHA + 0.5 * i.ax * ALPHA * ALPHA;
    i.y += i.vy * ALPHA + 0.5 * i.ay * ALPHA * ALPHA;
    i.vx += i.ax * ALPHA;
    i.vy += i.ay * ALPHA;
    if i.x + RADIUS >= rw {
        i.x = rw - RADIUS - EPSILON;
        i.vx = -0.5 * i.vx;
    }
    if i.x - RADIUS <= 0.0 {
        i.x = RADIUS + EPSILON;
        i.vx = -0.5 * i.vx;
    }
    if i.y + RADIUS >= rh {
        i.y = rh - RADIUS - EPSILON;
        i.vy = -0.5 * i.vy;
    }
    if i.y - RADIUS <= 0.0 {
        i.y = RADIUS + EPSILON;
        i.vy = -0.5 * i.vy;
    }
    i.ax = 0.0;
    i.ay = 0.0;
}