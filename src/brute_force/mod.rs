use rand::Rng;
use sdl2::event::Event;
use sdl2::pixels::Color;

use seq_module::*;

use crate::geometry::SimpleBody;
use crate::global::*;

mod seq_module;

fn init_universe() -> Vec<SimpleBody> {
    let mut universe = Vec::new();
    let mut rng = rand::thread_rng();
    let real_width = *WIDTH / *SCALE_FACTOR;
    let real_height = *HEIGHT / *SCALE_FACTOR;
    for _ in 0..*SIZE {
        universe.push(
            SimpleBody {
                x: rng.gen_range(0.0, real_width),
                y: rng.gen_range(0.0, real_height),
                m: rng.gen_range(0.0, MASS_RANGE),
                vx: 0.0,
                vy: 0.0,
                ax: 0.0,
                ay: 0.0,
            }
        );
    }
    universe
}

pub fn start_brute_force() {
    if *BENCHMARK {
        let mut universe = init_universe();
        let start = std::time::SystemTime::now();
        handle_impact(&mut universe);
        update_state(&mut universe);
        let end = std::time::SystemTime::now();
        println!("Duration: {} ms", end.duration_since(start).unwrap().as_millis());
    } else {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("HW3-Brute Force", *WIDTH as u32,
                                            *HEIGHT as u32)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        let mut universe = init_universe();
        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();
        canvas.present();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut i = 0;
        let mut n = 0;
        let mut start = std::time::SystemTime::now();
        'running: loop {
            n += 1;
            canvas.set_scale(*SCALE_FACTOR as f32, *SCALE_FACTOR as f32).unwrap();
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            canvas.clear();
            i = (i + 1) % 255;
            canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
            let points = universe.iter().map(|x| x.to_sdl()).collect::<Vec<_>>();
            canvas.draw_points(points.as_slice()).expect("unable to draw points");
            handle_impact(&mut universe);
            update_state(&mut universe);
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