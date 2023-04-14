use std::time::SystemTime;

use clap::*;
use hashbrown::HashMap;
use lazy_static;
use mpi::environment::*;
use mpi::topology::{Process, SystemCommunicator};
use mpi::traits::Communicator;
use nalgebra::Vector2;
use parking_lot::RwLock;
use crate::geometry::Point;

lazy_static! {
    pub static ref VMAP : RwLock<HashMap<Point, Vector2<f64>>> = RwLock::new(HashMap::new());

    pub static ref UNIVERSE : Universe = initialize().unwrap();

    pub static ref WORLD : SystemCommunicator = UNIVERSE.world();

    static ref ENGINES : Vec<&'static str> =
        vec!["tree", "openmp", "pthread", "mpi_normal", "mpi_openmp", "brute_force", "rayon", "rayon_tree"];

    static ref MODES : Vec<&'static str> =
        vec!["benchmark", "display"];

    pub static ref MATCHES : Option<ArgMatches<'static>> = {
        let result = App::new("Assignment-3")
        .version("2019Full-A3")
        .author("Schrodinger Zhu <i@zhuyi.fan>")
        .arg(Arg::with_name("engine")
            .short("e").value_name("ENGINE").help("render engine").required(true)
            .possible_values(ENGINES.as_slice()))
        .arg(Arg::with_name("width")
            .short("w").value_name("WIDTH").help("canvas width").default_value("1000"))
        .arg(Arg::with_name("height")
            .short("h").value_name("HEIGHT").help("canvas height").default_value("1000"))
        .arg(Arg::with_name("scale")
            .short("s").value_name("SCALE").help("scale factor").default_value("4.0"))
        .arg(Arg::with_name("number")
            .short("n").value_name("NUM").help("number of bodies").default_value("1000"))
        .arg(Arg::with_name("thread").help("thread number (for openmp/pthread), must be greater than 0, otherwise reset to 6")
            .short("t").default_value("6"))
        .arg(Arg::with_name("mode").value_name("MODE")
            .short("m").help("running mode").possible_values(MODES.as_slice()).default_value("display"))
        .arg(Arg::with_name("fps").value_name("FPS_FLAG")
            .short("f").help("whether to show fps").possible_values(&["yes", "no"]).default_value("no"))
        .get_matches_safe();
        match result {
            Ok(x) => Some(x),
            Err(m) => {
                if WORLD.rank() == ROOT {
                    m.exit();
                }
                None
            }
        }
    };

    pub static ref WIDTH : f64 = match MATCHES.as_ref().and_then(|m| m.value_of("width").and_then(|x|x.parse::<usize>().ok())) {
        Some(w) if w > 0 => w as f64,
        _ => 800.0
    };

    pub static ref HEIGHT : f64 = match MATCHES.as_ref().and_then(|m| m.value_of("height").and_then(|x|x.parse::<usize>().ok())) {
        Some(w) if w > 0 => w as f64,
        _ => 600.0
    };

    pub static ref SCALE_FACTOR : f64 = match MATCHES.as_ref().and_then(|m| m.value_of("scale").and_then(|x|x.parse::<f64>().ok())) {
        Some(w) if w > 0.0 => w,
        _ => 1.0
    };

    pub static ref SIZE : usize = match MATCHES.as_ref().and_then(|m| m.value_of("number").and_then(|x|x.parse::<usize>().ok())) {
        Some(w)  => w,
        _ => 50
    };

    pub static ref BENCHMARK : bool = match MATCHES.as_ref().and_then(|m| m.value_of("mode")) {
        Some("benchmark") => true,
        _ => false
    };

    pub static ref FPS_FLAG : bool = match MATCHES.as_ref().and_then(|m| m.value_of("fps")) {
        Some("yes") => true,
        _ => false
    };

    pub static ref THREAD : usize = match MATCHES.as_ref().and_then(|m| m.value_of("thread").and_then(|x|x.parse::<usize>().ok())) {
        Some(w) if w > 0 => w,
        _ => 6
    };

    // pub static ref ROOT_PROC : Process<'static, SystemCommunicator> =  WORLD.process_at_rank(ROOT);
}

pub const MIN_SIZE: f64 = 10.0;
pub const DIST_SCALE_LIMIT: f64 = 0.75;
pub const RADIUS: f64 = 0.5;
pub const G: f64 = 5.0;
pub const ALPHA: f64 = 0.001;
pub const ROOT: i32 = 0;
pub const MASS_RANGE: f64 = 50.0;

pub fn show_fps(n: &mut usize, start: &mut SystemTime) {
    if *FPS_FLAG {
        let cur = std::time::SystemTime::now();
        let du = cur.duration_since(*start).unwrap().as_millis();
        if du >= 1000 {
            println!("FPS: {}", *n as f64 / du as f64 * 1000.0);
            *start = cur;
            *n = 0;
        }
    }
}