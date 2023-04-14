#![recursion_limit = "512"]

#[macro_use]
extern crate cpp;
#[macro_use]
extern crate lazy_static;

use std::process::exit;

use mpi::traits::Communicator;

use global::MATCHES;

use crate::brute_force::start_brute_force;
use crate::mpi_eng::{start_mpi_child, start_mpi_root};
use crate::openmp::start_openmp;
use crate::pthread::start_thread_tree;
use crate::rayon_eng::start_rayon;
use crate::seq::start_tree;

mod seq;
mod openmp;
mod pthread;
mod mpi_eng;
mod brute_force;
mod rayon_eng;
pub mod global;
pub mod quad_tree;
pub mod geometry;

fn check_thread() {
    check_mpi();
    if *global::THREAD > *global::SIZE {
        if global::WORLD.rank() == global::ROOT {
            eprintln!("it is not reasonable to have more threads than bodies")
        }
        exit(0);
    }
}

fn check_mpi() {
    if global::WORLD.size() > 1 {
        if global::WORLD.rank() == global::ROOT {
            eprintln!("you should not use this engine with multiprocess");
        }
        exit(0);
    }
}

pub fn main() {
    let engine = MATCHES.as_ref().and_then(|m| m.value_of("engine"));
    engine.iter().for_each(|e| if global::WORLD.rank() == global::ROOT {
        print!("Name: Yifan ZHU\nStudent ID: 118010469\nAssignment 3, N-Body Simulation\n");
        println!("Engine: {}", e);
        println!("Scale Factor: {}", *global::SCALE_FACTOR);
        println!("Height: {}", *global::HEIGHT);
        println!("Width: {}", *global::WIDTH);
        println!("Size: {}", *global::SIZE);
        if *e == "openmp" || *e == "pthread" || *e == "mpi_openmp" {
            println!("Thread: {}", *global::THREAD);
        }
        if e.contains("mpi") {
            println!("Process: {}", global::WORLD.size());
        }
    });
    match engine {
        Some("tree") => {
            check_mpi();
            start_tree();
        }
        Some("brute_force") => {
            check_mpi();
            start_brute_force();
        }
        Some("openmp") => {
            check_thread();
            start_openmp();
        }
        Some("rayon") => {
            check_mpi();
            start_rayon();
        }
        Some("rayon_tree") => {
            check_mpi();
            start_thread_tree(true);
        }
        Some("pthread") => {
            check_thread();
            start_thread_tree(false);
        }
        Some("mpi_normal") => {
            let world_size = global::WORLD.size() as usize;
            if world_size > *global::SIZE {
                if global::WORLD.rank() == global::ROOT {
                    println!("it is not reasonable to have more processes than bodies")
                }
            } else {
                if global::WORLD.rank() == global::ROOT {
                    start_mpi_root(false)
                } else {
                    start_mpi_child(false)
                }
            }
        }
        Some("mpi_openmp") => {
            let world_size = global::WORLD.size() as usize;
            if world_size > *global::SIZE {
                if global::WORLD.rank() == global::ROOT {
                    println!("it is not reasonable to have more processes than bodies")
                }
            } else {
                if global::WORLD.rank() == global::ROOT {
                    start_mpi_root(true)
                } else {
                    start_mpi_child(true)
                }
            }
        }
        _ => ()
    }
}