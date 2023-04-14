use rand::Rng;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Point;

use crate::global;
use crate::global::MASS_RANGE;
use crate::openmp::cpp_module::{handle_collision, setup, update_acc, update_state};

pub mod cpp_module;

fn to_sdl(x: &[f64], y: &[f64]) -> Vec<Point> {
    let mut a = Vec::new();
    for i in 0..x.len() {
        a.push(Point::new(x[i] as i32, y[i] as i32))
    }
    a
}

fn benchmark_mode() {
    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut vx = Vec::new();
    let mut vy = Vec::new();
    let mut ax = Vec::new();
    let mut ay = Vec::new();
    let mut m = Vec::new();
    let real_width = *global::WIDTH / *global::SCALE_FACTOR;
    let real_height = *global::HEIGHT / *global::SCALE_FACTOR;

    let mut rng = rand::thread_rng();
    for _ in 0..*global::SIZE {
        x.push(rng.gen_range(0.0, real_width - global::RADIUS));
        y.push(rng.gen_range(0.0, real_height - global::RADIUS));
        m.push(rng.gen_range(0.0, MASS_RANGE));
        ax.push(0.0);
        ay.push(0.0);
        vx.push(0.0);
        vy.push(0.0);
    }
    let start = std::time::SystemTime::now();
    handle_collision(&m, &mut vx, &mut vy, &mut x, &mut y, 0, *global::SIZE);
    update_acc(&m, &mut x, &mut y, &mut ax, &mut ay, 0, *global::SIZE);
    update_state(&mut x, &mut y, &mut ax, &mut ay, &mut vx, &mut vy, 0, *global::SIZE);
    let end = std::time::SystemTime::now();
    println!("Duration: {} ms", end.duration_since(start).unwrap().as_millis());
}

pub fn start_openmp() {
    setup();
    if *global::BENCHMARK {
        return benchmark_mode();
    }
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("HW3-OpenMP", *global::WIDTH as u32,
                                        *global::HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut vx = Vec::new();
    let mut vy = Vec::new();
    let mut ax = Vec::new();
    let mut ay = Vec::new();
    let mut m = Vec::new();
    let real_width = *global::WIDTH / *global::SCALE_FACTOR;
    let real_height = *global::HEIGHT / *global::SCALE_FACTOR;

    let mut rng = rand::thread_rng();
    for _ in 0..*global::SIZE {
        x.push(rng.gen_range(0.0, real_width - global::RADIUS));
        y.push(rng.gen_range(0.0, real_height - global::RADIUS));
        m.push(rng.gen_range(0.0, MASS_RANGE));
        ax.push(0.0);
        ay.push(0.0);
        vx.push(0.0);
        vy.push(0.0);
    }

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    let mut n = 0;
    let mut start = std::time::SystemTime::now();
    'running: loop {
        n += 1;
        canvas.set_scale(*global::SCALE_FACTOR as f32, *global::SCALE_FACTOR as f32).unwrap();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        let points = to_sdl(x.as_slice(), y.as_slice());
        canvas.draw_points(points.as_slice()).expect("unable to draw points");
        handle_collision(&m, &mut vx, &mut vy, &mut x, &mut y, 0, *global::SIZE);
        update_acc(&m, &mut x, &mut y, &mut ax, &mut ay, 0, *global::SIZE);
        update_state(&mut x, &mut y, &mut ax, &mut ay, &mut vx, &mut vy, 0, *global::SIZE);
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'running;
                }
                _ => {}
            }
        }
        canvas.present();
        global::show_fps(&mut n, &mut start);
    }
}

