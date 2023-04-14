use std::sync::Arc;

use nalgebra::Vector2;
use rand::Rng;
use sdl2::event::Event;
use sdl2::pixels::Color;

use crate::geometry;
use crate::geometry::{Body, Square};
use crate::global;
use crate::quad_tree;
use crate::quad_tree::node::QuadNode;
use std::f64::EPSILON;

fn refresh(pool: &mut Vec<Body>, root: &mut Arc<QuadNode>, boundary: &Square) {
    {
        let mut a = global::VMAP.write();
        a.clear();
        for i in &*pool {
            a.insert(i.position.clone(), i.velocity);
        }
        for i in &*pool {
            assert!(a.contains_key(&i.position))
        }
    }
    for i in &mut *pool {
        i.make_ready();
        i.collision_detect();
        i.update_velocity();
        i.update_position();
        i.gravity_impact(root.clone());
        i.check_boundary();
    }
    *root = Arc::new(quad_tree::node::QuadNode::new(boundary.clone()));
    for i in &mut *pool {
        i.reinsert(root.clone());
    }
}

pub fn start_tree() {
    let real_width = *global::WIDTH / *global::SCALE_FACTOR;
    let real_height = *global::HEIGHT / *global::SCALE_FACTOR;
    let boundary = geometry::Square(
        Vector2::new(real_width, real_height),
        Vector2::new(0.0, 0.0)
    );

    let mut root = Arc::new(quad_tree::node::QuadNode::new(boundary.clone()));
    let mut rng = rand::thread_rng();
    let mut pool = Vec::new();
    for _ in 0..*global::SIZE {
        pool.push(geometry::Body::new(
            rng.gen_range(global::RADIUS + EPSILON, real_width - global::RADIUS),
            rng.gen_range(global::RADIUS + EPSILON, real_height - global::RADIUS),
            rng.gen_range(0.0, 20.0),
            root.clone(),
        ));
    }
    if *global::BENCHMARK {
        let start = std::time::SystemTime::now();
        refresh(&mut pool, &mut root, &boundary);
        let end = std::time::SystemTime::now();
        println!("Duration: {} ms", end.duration_since(start).unwrap().as_millis());
    } else {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("HW3-Sequential", *global::WIDTH as u32,
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
            //println!("{:?}", pool);
            canvas.set_scale(*global::SCALE_FACTOR as f32, *global::SCALE_FACTOR as f32).unwrap();
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            canvas.clear();
            i = (i + 1) % 255;
            canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
            let points = pool.iter().map(|x| x.geometric()).collect::<Vec<_>>();
            canvas.draw_points(points.as_slice()).expect("unable to draw points");

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => {
                        break 'running;
                    }
                    _ => {}
                }
            }

            canvas.present();
            refresh(&mut pool, &mut root, &boundary);
            global::show_fps(&mut n, &mut start);
        }
    }
}