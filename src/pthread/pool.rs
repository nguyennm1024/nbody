use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;

use nalgebra::Vector2;
use rayon::prelude::*;
use sdl2::rect::Point;

use crate::geometry;
use crate::geometry::Body;
use crate::global::{HEIGHT, SCALE_FACTOR, SIZE, THREAD, WIDTH};
use crate::quad_tree::node::QuadNode;

struct SharedData {
    root: Arc<QuadNode>,
    finished: AtomicUsize,
}

pub fn new_root() -> Arc<QuadNode> {
    let real_width = *WIDTH / *SCALE_FACTOR;
    let real_height = *HEIGHT / *SCALE_FACTOR;
    let boundary = geometry::Square(
        Vector2::new(real_width, real_height),
        Vector2::new(0.0, 0.0)
    );

    Arc::new(QuadNode::new(boundary))
}

fn chunk_size(total: usize, group: usize, kth: usize) -> usize {
    let a = total - kth;
    if a % group > 0 { a / group + 1 } else { a / group }
}

pub struct BodyWrapper {
    ptr: Arc<RefCell<geometry::Body>>
}

unsafe impl Sync for BodyWrapper {}

unsafe impl Send for BodyWrapper {}

impl Clone for BodyWrapper {
    fn clone(&self) -> Self {
        BodyWrapper {
            ptr: self.ptr.clone()
        }
    }
}

impl From<Body> for BodyWrapper {
    fn from(body: Body) -> Self {
        BodyWrapper {
            ptr: Arc::new(RefCell::new(body))
        }
    }
}

impl BodyWrapper {
    pub(crate) fn to_sdl(&self) -> sdl2::rect::Point {
        let body = self.ptr.borrow();
        Point::new(
            body.position.x as i32,
            body.position.y as i32,
        )
    }
}

pub fn thread_go(points: &Vec<BodyWrapper>, last_root: Arc<QuadNode>) -> Arc<QuadNode> {
    crate::global::VMAP.write().clear();
    let shared = Arc::new(SharedData {
        root: new_root(),
        finished: AtomicUsize::new(0),
    });
    let mut counter = 0;
    {
        let mut lock = crate::global::VMAP.write();
        for i in points {
            let mut inst = i.ptr.borrow_mut();
            inst.make_ready();
            lock.insert(inst.position.clone(), inst.velocity.clone());
        }
    }
    for i in 0..*THREAD {
        let shared = shared.clone();
        let work_size = chunk_size(*SIZE, *THREAD, i);
        let points = (&points[counter..counter + work_size])
            .iter().map(|x| x.clone()).collect::<Vec<_>>();
        let last_root = last_root.clone();
        std::thread::spawn(move || {
            for i in &points {
                let mut instance = i.ptr.borrow_mut();
                instance.collision_detect();
                instance.update_velocity();
                instance.update_position();
                instance.check_boundary();
                instance.gravity_impact(last_root.clone());
                instance.reinsert(shared.root.clone());
            }
            shared.finished.fetch_add(1, SeqCst);
        });
        counter += work_size
    }
    while shared.finished.load(SeqCst) < *THREAD {
        std::thread::yield_now();
    }
    shared.root.clone()
}

pub fn thread_rayon(points: &Vec<BodyWrapper>, last_root: Arc<QuadNode>) -> Arc<QuadNode> {
    crate::global::VMAP.write().clear();
    let shared = Arc::new(SharedData {
        root: new_root(),
        finished: AtomicUsize::new(0),
    });

    {
        let mut lock = crate::global::VMAP.write();
        for i in points {
            let mut inst = i.ptr.borrow_mut();
            inst.make_ready();
            lock.insert(inst.position.clone(), inst.velocity.clone());
        }
    }

    points.par_iter().for_each(|i| {
        let mut inst = i.ptr.borrow_mut();
        inst.make_ready();
    });

    points.par_iter().for_each(|i| {
        let mut instance = i.ptr.borrow_mut();
        instance.collision_detect();
        instance.update_velocity();
        instance.update_position();
        instance.check_boundary();
        instance.gravity_impact(last_root.clone());
        instance.reinsert(shared.root.clone());
    });

    shared.root.clone()
}