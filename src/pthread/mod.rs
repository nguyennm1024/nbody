use rand::Rng;
use sdl2::event::Event;
use sdl2::pixels::Color;

use crate::geometry::Body;
use crate::global;
use crate::global::{MASS_RANGE, RADIUS};
use crate::pthread::pool::*;
use std::f64::EPSILON;

pub mod pool;

pub fn start_thread_tree(with_rayon: bool) {
    let real_width = *global::WIDTH / *global::SCALE_FACTOR;
    let real_height = *global::HEIGHT / *global::SCALE_FACTOR;
    let mut body_wrappers = Vec::new();
    let mut rng = rand::thread_rng();
    let mut root = pool::new_root();

    for _ in 0..*global::SIZE {
        let body = Body::new(
            rng.gen_range(RADIUS + EPSILON, real_width),
            rng.gen_range(RADIUS + EPSILON, real_height),
            rng.gen_range(0.0, MASS_RANGE),
            root.clone(),
        );
        body_wrappers.push(BodyWrapper::from(body));
    }
    if *global::BENCHMARK {
        let start = std::time::SystemTime::now();
        if with_rayon {
            thread_rayon(&body_wrappers, root);
        } else {
            thread_go(&body_wrappers, root);
        }
        let end = std::time::SystemTime::now();
        println!("Duration: {} ms", end.duration_since(start).unwrap().as_millis());
    } else {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window(if with_rayon { "HW3-RayonTree" } else { "HW3-PThread" }, *global::WIDTH as u32,
                                            *global::HEIGHT as u32)
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();


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
            let points = body_wrappers.iter().map(|x| x.to_sdl()).collect::<Vec<_>>();
            canvas.draw_points(points.as_slice()).expect("unable to draw points");
            if with_rayon {
                root = thread_rayon(&body_wrappers, root);
            } else {
                root = thread_go(&body_wrappers, root);
            }
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => {
                        break 'running;
                    }
                    _ => {}
                }
            }
            canvas.present();
            crate::global::show_fps(&mut n, &mut start);
        }
    }
}

