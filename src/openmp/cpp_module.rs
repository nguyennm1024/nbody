use std::f64::EPSILON;

use cpp;

use crate::global::*;

cpp! {{
#include <cmath>
#include <omp.h>
#include <vector>
#define dist_squared(i, j)  ((x_pos[(i)] - x_pos[(j)]) * (x_pos[(i)] - x_pos[(j)]) + (y_pos[(i)] - y_pos[(j)]) * (y_pos[(i)] - y_pos[(j)]))
#define check(i, j) (dist_squared(i, j) <= 4.0 * (radius) * (radius))
}}

cpp! {{
#define cross(i, j) (((x_pos[(i)] - x_pos[(j)]) * (vx[(i)] - vx[(j)])) + ((y_pos[(i)] - y_pos[(j)]) * (vy[(i)] - vy[(j)])))
#define coefficient(i, j) (2.0 * mass[(j)] / (mass[(i)] + mass[(j)]))
}}

cpp! {{
#define update_vx(i, j) (impact_x[(i) - from] -= coefficient(i, j) * cross(i, j) / dist_squared(i, j) * (x_pos[(i)] - x_pos[(j)]))
#define update_vy(i, j) (impact_y[(i) - from] -= coefficient(i, j) * cross(i, j) / dist_squared(i, j) * (y_pos[(i)] - y_pos[(j)]))
#define update_v(i, j) ((update_vx((i), (j))), (update_vy((i), (j))))
}}

cpp! {{
#define scale(i, j)  (g * mass[(j)] / (dist_squared((i), (j)) * sqrt(dist_squared((i), (j)))))
#define update_a(i, j) ((ax[i] = scale(i, j) * (x_pos[j] - x_pos[i])), (ay[i] = scale(i, j) * (y_pos[j] - y_pos[i])))
}}

pub fn setup() {
    let thn = *THREAD as i32;
    unsafe {
        cpp!([thn as "int"] -> () as "void" {
            omp_set_num_threads(thn);
        })
    }
}


pub fn handle_collision(mass: &[f64],
                        vx: &mut [f64],
                        vy: &mut [f64],
                        x_pos: &mut [f64],
                        y_pos: &mut [f64],
                        from: usize,
                        to: usize,
) {
    unsafe {
        let size = *SIZE;
        let radius = RADIUS;
        let mass = mass.as_ptr();
        let vx = vx.as_mut_ptr();
        let vy = vy.as_mut_ptr();
        let x_pos = x_pos.as_mut_ptr();
        let y_pos = y_pos.as_mut_ptr();
        cpp!(
            [mass as "const double *",
            size as "size_t", radius as "double",
            x_pos as "double *", y_pos as "double *",
            vx as "double *", vy as "double *", from as "size_t", to as "size_t"] -> () as "void" {
                std::vector<double> impact_x(to - from, 0);
                std::vector<double> impact_y(to - from, 0);
                #pragma omp parallel for schedule(guided)
                for (size_t i = from; i < to; ++i) {
                    for (size_t j = 0; j < size; ++j) {
                        if (i == j) continue;
                        else if (check(i, j)) {update_v(i, j); }
                    }
                }
                #pragma omp parallel for schedule(guided)
                for (size_t i = from; i < to; ++i) {
                    vx[i] += impact_x[i - from];
                    vy[i] += impact_y[i - from];
                }
            }
        )
    }
}


pub fn update_state(x_pos: &mut [f64],
                    y_pos: &mut [f64],
                    ax: &mut [f64],
                    ay: &mut [f64],
                    vx: &mut [f64],
                    vy: &mut [f64],
                    from: usize,
                    to: usize,
) {
    unsafe {
        let radius = RADIUS;
        let width = *WIDTH / *SCALE_FACTOR;
        let height = *HEIGHT / *SCALE_FACTOR;
        let eps = EPSILON;
        let ax = ax.as_mut_ptr();
        let ay = ay.as_mut_ptr();
        let vx = vx.as_mut_ptr();
        let vy = vy.as_mut_ptr();
        let x_pos = x_pos.as_mut_ptr();
        let y_pos = y_pos.as_mut_ptr();
        let alpha = ALPHA;
        cpp!(
            [radius as "double", alpha as "double", width as "double", height as "double",
            x_pos as "double *", y_pos as "double *", eps as "double", from as "size_t", to as "size_t",
            ax as "double *", ay as "double *", vx as "double *", vy as "double *"] -> () as "void" {
                #pragma omp parallel for schedule(guided)
                for (size_t i = from; i < to; ++i) {
                    if (std::isnan(vx[i])) {
                        vx[i] = 0;
                        x_pos[i] = 0.618 * width;
                    }
                    if (std::isnan(vy[i])) {
                        vy[i] = 0;
                        y_pos[i] = 0.618 * height;
                    }
                    vx[i] += ax[i] * alpha;
                    vy[i] += ay[i] * alpha;
                    x_pos[i] += vx[i] * alpha + 0.5 * ax[i] * alpha * alpha;
                    y_pos[i] += vy[i] * alpha + 0.5 * ay[i] * alpha * alpha;
                    if (x_pos[(i)] + radius >= width) { x_pos[(i)] = width - radius - eps; vx[(i)] = -0.5 * vx[(i)]; }
                    if (x_pos[(i)] - radius <= 0) { x_pos[(i)] = radius + eps;  vx[(i)] = -0.5 * vx[(i)]; }
                    if (y_pos[(i)] + radius >= height) { y_pos[(i)] = height - radius - eps; vy[(i)] = -0.5 * vy[(i)]; }
                    if (y_pos[(i)] - radius <= 0) { y_pos[(i)] = radius + eps; vy[(i)] = -0.5 * vy[(i)]; }
                }
            }
        )
    }
}

pub fn update_acc(mass: &[f64],
                  x_pos: &mut [f64],
                  y_pos: &mut [f64],
                  ax: &mut [f64],
                  ay: &mut [f64],
                  from: usize,
                  to: usize,
) {
    unsafe {
        let size = *SIZE;
        let radius = RADIUS;
        let g = G;
        let mass = mass.as_ptr();
        let ax = ax.as_mut_ptr();
        let ay = ay.as_mut_ptr();
        let x_pos = x_pos.as_mut_ptr();
        let y_pos = y_pos.as_mut_ptr();
        cpp!(
            [mass as "const double *",
            size as "size_t", radius as "double", g as "double",
            x_pos as "double *", y_pos as "double *", from as "size_t", to as "size_t",
            ax as "double *", ay as "double *"] -> () as "void" {
                #pragma omp parallel for schedule(guided)
                for (size_t i = from; i < to; ++i) {
                    for (size_t j = 0; j < size; ++j) {
                        if (check(i, j)) {continue; }
                        else {
                            update_a(i, j);
                        }
                    }
                }
            }
        )
    }
}


