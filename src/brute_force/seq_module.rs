use std::f64::EPSILON;

use crate::geometry::SimpleBody;
use crate::global::*;

pub fn handle_impact(universe: &mut Vec<SimpleBody>) {
    let universe_size = universe.len();
    for i in 0..universe_size {
        for j in i + 1..universe_size {
            let delta_x = universe[i].x - universe[j].x;
            let delta_y = universe[i].y - universe[j].y;
            let dist = delta_x * delta_x + delta_y * delta_y;
            if dist <= RADIUS * RADIUS * 4.0 {
                let dot = delta_x * (universe[i].vx - universe[j].vx)
                    + delta_y * (universe[i].vy - universe[j].vy);
                let scale = 2.0 / (universe[i].m + universe[j].m) * dot / dist;
                universe[i].vx -= scale * delta_x * universe[j].m;
                universe[i].vy -= scale * delta_y * universe[j].m;
                universe[j].vx += scale * delta_x * universe[i].m;
                universe[j].vy += scale * delta_y * universe[i].m;
            } else {
                let scale = G / dist / dist.sqrt();
                universe[i].ax -= delta_x * scale * universe[j].m;
                universe[i].ay -= delta_y * scale * universe[j].m;
                universe[j].ax += delta_x * scale * universe[i].m;
                universe[j].ay += delta_y * scale * universe[i].m;
            }
        }
    }
}

pub fn update_state(universe: &mut Vec<SimpleBody>) {
    for i in universe {
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
}