use std::f64::EPSILON;

use mpi::traits::*;
use rand::Rng;

use crate::global::*;
use crate::openmp::cpp_module::*;

pub struct GlobalData {
    gx: Vec<f64>,
    gy: Vec<f64>,
    gvx: Vec<f64>,
    gvy: Vec<f64>,
    gax: Vec<f64>,
    gay: Vec<f64>,
    m: Vec<f64>,
}


impl GlobalData {
    pub fn new() -> Self {
        let world_size = WORLD.size() as usize;
        let size = world_size * if *SIZE % world_size > 0 { *SIZE / world_size + 1 } else { *SIZE / world_size };

        let mut res = GlobalData {
            gx: Vec::with_capacity(size),
            gy: Vec::with_capacity(size),
            gvx: Vec::with_capacity(size),
            gvy: Vec::with_capacity(size),
            gax: Vec::with_capacity(size),
            gay: Vec::with_capacity(size),
            m: Vec::with_capacity(size),
        };
        if WORLD.rank() != ROOT {
            res.m.resize(size, 0.0);
            res.gx.resize(size, 0.0);
            res.gy.resize(size, 0.0);
            res.gvx.resize(size, 0.0);
            res.gvy.resize(size, 0.0);
            res.gax.resize(size, 0.0);
            res.gay.resize(size, 0.0);
            res.m.resize(size, 0.0);
        } else {
            let mut rng = rand::thread_rng();
            for _ in 0..size {
                let real_width = *WIDTH / *SCALE_FACTOR;
                let real_height = *HEIGHT / *SCALE_FACTOR;
                res.gx.push(rng.gen_range(0.0, real_width - RADIUS));
                res.gy.push(rng.gen_range(0.0, real_height - RADIUS));
                res.m.push(rng.gen_range(0.0, MASS_RANGE));
                res.gax.push(0.0);
                res.gay.push(0.0);
                res.gvx.push(0.0);
                res.gvy.push(0.0);
            }
        }
        ROOT_PROC.broadcast_into(res.m.as_mut_slice());
        res
    }
    pub fn broadcast(&mut self) {
        ROOT_PROC.broadcast_into(self.gx.as_mut_slice());
        ROOT_PROC.broadcast_into(self.gy.as_mut_slice());
        ROOT_PROC.broadcast_into(self.gvx.as_mut_slice());
        ROOT_PROC.broadcast_into(self.gvy.as_mut_slice());
    }
    fn update_impact(&mut self, k: usize, x_buffer: &mut Vec<f64>, y_buffer: &mut Vec<f64>, iter: usize) {
        let mut ax_acc = 0.0;
        let mut ay_acc = 0.0;
        for i in 0..*SIZE {
            if i == k { continue; } else {
                let dist_squared = (self.gx[k] - self.gx[i]) * (self.gx[k] - self.gx[i]) + (self.gy[k] - self.gy[i]) * (self.gy[k] - self.gy[i]);
                if dist_squared > 4.0 * RADIUS * RADIUS {
                    let scale = G * self.m[i] / dist_squared / dist_squared.sqrt();
                    ax_acc += scale * (self.gx[i] - self.gx[k]);
                    ay_acc += scale * (self.gy[i] - self.gy[k]);
                } else {
                    let delta_x = self.gx[k] - self.gx[i];
                    let delta_y = self.gy[k] - self.gy[i];
                    let dot = delta_x * (self.gvx[k] - self.gvx[i]) + delta_y * (self.gvy[k] - self.gvy[i]);
                    let scale = 2.0 * self.m[i] / (self.m[i] + self.m[k]) * dot / dist_squared;
                    x_buffer[iter] -= scale * delta_x;
                    y_buffer[iter] -= scale * delta_y;
                }
            }
        }
        self.gax[k] = ax_acc;
        self.gay[k] = ay_acc;
    }
    fn update_state(&mut self, i: usize) {
        let rw: f64 = *WIDTH / *SCALE_FACTOR;
        let rh: f64 = *HEIGHT / *SCALE_FACTOR;
        if self.gvx[i].is_nan() {
            self.gvx[i] = 0.0;
            self.gx[i] = 0.618 * *WIDTH / *SCALE_FACTOR;
        }
        if self.gvy[i].is_nan() {
            self.gvy[i] = 0.0;
            self.gy[i] = 0.618 * *HEIGHT / *SCALE_FACTOR;
        }
        self.gx[i] += self.gvx[i] * ALPHA + 0.5 * self.gax[i] * ALPHA * ALPHA;
        self.gy[i] += self.gvy[i] * ALPHA + 0.5 * self.gay[i] * ALPHA * ALPHA;
        self.gvx[i] += self.gax[i] * ALPHA;
        self.gvy[i] += self.gay[i] * ALPHA;
        if self.gx[i] + RADIUS >= rw {
            self.gx[i] = rw - RADIUS - EPSILON;
            self.gvx[i] = -0.5 * self.gvx[i];
        }
        if self.gx[i] - RADIUS <= 0.0 {
            self.gx[i] = RADIUS + EPSILON;
            self.gvx[i] = -0.5 * self.gvx[i];
        }
        if self.gy[i] + RADIUS >= rh {
            self.gy[i] = rh - RADIUS - EPSILON;
            self.gvy[i] = -0.5 * self.gvy[i];
        }
        if self.gy[i] - RADIUS <= 0.0 {
            self.gy[i] = RADIUS + EPSILON;
            self.gvy[i] = -0.5 * self.gvy[i];
        }
    }
    pub fn update_all_openmp(&mut self, s: usize, t: usize) {
        handle_collision(self.m.as_slice(),
                         self.gvx.as_mut_slice(),
                         self.gvy.as_mut_slice(),
                         self.gx.as_mut_slice(),
                         self.gy.as_mut_slice(), s, t);
        update_acc(self.m.as_slice(),
                   self.gx.as_mut_slice(),
                   self.gy.as_mut_slice(),
                   self.gax.as_mut_slice(),
                   self.gay.as_mut_slice(), s, t);
        update_state(self.gx.as_mut_slice(),
                     self.gy.as_mut_slice(),
                     self.gax.as_mut_slice(),
                     self.gay.as_mut_slice(),
                     self.gvx.as_mut_slice(),
                     self.gvy.as_mut_slice(), s, t);
    }
    pub fn update_all(&mut self, s: usize, t: usize) {
        let mut x_buffer = Vec::new();
        let mut y_buffer = Vec::new();
        let mut k = 0;
        x_buffer.resize(t - s, 0.0);
        y_buffer.resize(t - s, 0.0);
        for i in s..t {
            self.update_impact(i, &mut x_buffer, &mut y_buffer, k);
            k += 1;
        }
        k = 0;
        for i in s..t {
            self.gvx[i] += x_buffer[k];
            self.gvy[i] += y_buffer[k];
            self.update_state(i);
            k += 1;
        }
    }
    pub fn to_sdl(&self, starts: &Vec<usize>, ends: &Vec<usize>) -> Vec<sdl2::rect::Point> {
        let mut a = Vec::new();
        for i in 0..starts.len() {
            for j in starts[i]..ends[i] {
                a.push(sdl2::rect::Point::new(self.gx[j] as i32, self.gy[j] as i32))
            }
        }
        a
    }

    pub fn gather(&mut self, s: usize, t: usize) {
        let mut buffer = Vec::with_capacity(t - s);
        buffer.resize(t - s, 0.0);
        if WORLD.rank() == ROOT {
            buffer.copy_from_slice(self.gx[s..t].as_ref());
            ROOT_PROC.gather_into_root(buffer.as_slice(), self.gx.as_mut_slice());
            buffer.copy_from_slice(self.gy[s..t].as_ref());
            ROOT_PROC.gather_into_root(buffer.as_slice(), self.gy.as_mut_slice());
            buffer.copy_from_slice(self.gvx[s..t].as_ref());
            ROOT_PROC.gather_into_root(buffer.as_slice(), self.gvx.as_mut_slice());
            buffer.copy_from_slice(self.gvy[s..t].as_ref());
            ROOT_PROC.gather_into_root(buffer.as_slice(), self.gvy.as_mut_slice());
        } else {
            buffer.copy_from_slice(self.gx[s..t].as_ref());
            ROOT_PROC.gather_into(buffer.as_slice());
            buffer.copy_from_slice(self.gy[s..t].as_ref());
            ROOT_PROC.gather_into(buffer.as_slice());
            buffer.copy_from_slice(self.gvx[s..t].as_ref());
            ROOT_PROC.gather_into(buffer.as_slice());
            buffer.copy_from_slice(self.gvy[s..t].as_ref());
            ROOT_PROC.gather_into(buffer.as_slice());
        }
    }
}

pub fn chunk_size(total: usize, group: usize, kth: usize) -> (usize, bool) {
    let a = total - kth;
    let flag = kth >= total % group && total % group > 0;
    if a % group > 0 { (a / group + 1, flag) } else { (a / group, flag) }
}