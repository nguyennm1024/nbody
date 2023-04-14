use std::collections::HashSet;
use std::sync::{Arc, atomic::Ordering, Weak};
use std::sync::atomic::{AtomicU8, AtomicUsize};
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use parking_lot::{RwLock, Mutex};
use nalgebra::Vector2;

use crate::geometry::*;
use crate::global::*;
use std::cell::RefCell;

type Ptr = Arc<QuadNode>;


pub struct QuadNode {
    region: Square,
    objects: RwLock<HashSet<Point>>,
    children: [RwLock<Option<Arc<QuadNode>>>; 4],
    active: AtomicU8,
    parent: Option<Weak<QuadNode>>,
    size: AtomicUsize,
    mass: Arc<RefCell<f64>>,
    mass_center: Arc<RefCell<Vector2<f64>>>,
    mass_reader: *const f64,
    mass_center_reader: *const Vector2<f64>,
    _lock: Mutex<()>
}
unsafe impl Send for QuadNode {}
unsafe impl Sync for QuadNode {}


impl QuadNode {
    pub fn new(region: Square) -> Self {
        let mut res = QuadNode {
            region,
            objects: RwLock::new(HashSet::new()),
            children: [RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None)],
            active: AtomicU8::new(0),
            parent: None,
            size: AtomicUsize::new(0),
            mass: Arc::new(RefCell::new(0.0)),
            mass_center: Arc::new(RefCell::new(Vector2::new(0.0, 0.0))),
            _lock: Mutex::new(()),
            mass_reader: std::ptr::null(),
            mass_center_reader: std::ptr::null()
        };
        res.mass_center_reader = res.mass_center.as_ptr() as _;
        res.mass_reader = res.mass.as_ptr()  as _;
        res
    }
    pub fn new_parented(region: Square, pa: &Ptr) -> Self {
        let mut res = QuadNode {
            region,
            objects: RwLock::new(HashSet::new()),
            children: [RwLock::new(None), RwLock::new(None), RwLock::new(None), RwLock::new(None)],
            active: AtomicU8::new(0),
            parent: Some(Arc::downgrade(&pa)),
            size: AtomicUsize::new(0),
            mass: Arc::new(RefCell::new(0.0)),
            mass_center: Arc::new(RefCell::new(Vector2::new(0.0, 0.0))),
            _lock: Mutex::new(()),
            mass_reader: std::ptr::null(),
            mass_center_reader: std::ptr::null()
        };
        res.mass_center_reader = res.mass_center.as_ptr() as _;
        res.mass_reader = res.mass.as_ptr()  as _;
        res
    }
}

fn build(node: Ptr) {
    {
        let _lock = node.objects.read();
        node.size.store(_lock.len(), SeqCst);
        if _lock.len() <= 1 {
            let __lock = node._lock.lock();
            let mut mass = node.mass.borrow_mut();
            let mut mc = node.mass_center.borrow_mut();
            for i in _lock.iter() {
                *mass += i.mass;
                *mc += i.coords() * i.mass;
            }
            return;
        }
    }

    let range: Vector2<f64> = node.region.0 - node.region.1;
    if range.x <= MIN_SIZE && range.y <= MIN_SIZE {
        return;
    }

    let quadrant = area(&node);

    let mut quad_list = [
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];

    let mut del_list = Vec::new();

    {
        let __lock = node._lock.lock();
        let mut mass = node.mass.borrow_mut();
        let mut mc = node.mass_center.borrow_mut();
        for i in node.objects.read().iter() {
            *mass += i.mass;
            *mc += i.coords() * i.mass;
            for j in 0_usize..4_usize {
                if quadrant[j].contains(i) {
                    quad_list[j].push(i.clone());
                    del_list.push(i.clone());
                    break;
                }
            }
        }
    }

    {
        let mut _ref = node.objects.write();
        for i in del_list {
            _ref.remove(&i);
        }
    }

    for i in 0..4 {
        if quad_list[i].len() > 0 {
            {
                let mut child = node.children[i].write();
                child.replace(Arc::new(
                    QuadNode::new_parented(quadrant[i], &node)
                ));
            }
            node.active.fetch_or(1_u8 << i, Ordering::SeqCst);

            while let Some(a) = quad_list[i].pop() {
                node.children[i].read()
                    .as_ref().unwrap().objects.write().insert(a);
            }
            build(node.children[i].read().as_ref().unwrap().clone());
        }
    }
}

fn area(node: &Ptr) -> [Square; 4] {
    let range: Vector2<f64> = node.region.0 - node.region.1;
    let half: Vector2<f64> = range.scale(0.5);
    let center: Vector2<f64> = node.region.1 + half;
    [
        Square(center, node.region.1),
        Square(node.region.0, center),
        Square(Vector2::new(center.x, node.region.0.y), Vector2::new(node.region.1.x, center.y)),
        Square(Vector2::new(node.region.0.x, center.y), Vector2::new(center.x, node.region.1.y)),
    ]
}

pub fn insert(node: Ptr, p: Point) -> Arc<QuadNode> {
    if node.size.load(SeqCst) == 0 {
        {
            let __lock = node._lock.lock();
            node.objects.write().insert(p);
            node.size.fetch_add(1, SeqCst);
            *node.mass_center.borrow_mut() += p.coords() * p.mass;
            *node.mass.borrow_mut() += p.mass;
        }
        return node;
    }


    let range: Vector2<f64> = node.region.0 - node.region.1;
    if range.x <= MIN_SIZE && range.y <= MIN_SIZE {
        //println!("reached");
        {
            let __lock = node._lock.lock();
            node.objects.write().insert(p);
            node.size.fetch_add(1, SeqCst);
            *node.mass.borrow_mut() += p.mass;
            *node.mass_center.borrow_mut() += p.coords() * p.mass;
        }
        return node;
    }

    let quadrant = area(&node);

    let mut flag = false;
    let mut res = node.clone();
    for i in 0..4 {
        if quadrant[i].contains(&p) {
            flag = true;
            let mut _lock = node.children[i].write();
            if let Some(child) = _lock.as_ref() {
                let __lock = node._lock.lock();
                node.size.fetch_add(1, SeqCst);
                *node.mass_center.borrow_mut() += p.coords() * p.mass;
                *node.mass.borrow_mut() += p.mass;
                return insert(child.clone(), p);
            } else {
                res = Arc::new(QuadNode::new_parented(quadrant[i].clone(), &node));
                res.objects.write().insert(p);
                build(res.clone());
                _lock.replace(res.clone());
                node.active.fetch_or(1_u8 << i, Ordering::SeqCst);
            }
            break;
        }
    }

    if !flag {
        node.objects.write().insert(p);
    }
    let __lock = node._lock.lock();
    node.size.fetch_add(1, SeqCst);
    *node.mass.borrow_mut() += p.mass;
    *node.mass_center.borrow_mut() += p.coords() * p.mass;
    res
}

pub fn make_ready(p: Point, node: Arc<QuadNode>) -> Arc<QuadNode> {
    if node.active.load(SeqCst) == 0 {
        return node;
    }
    let mut res = node.clone();
    let quadrant = area(&node);
    for i in 0..4 {
        if quadrant[i].contains(&p) {
            let mut _lock = node.children[i].write();
            if let Some(child) = _lock.as_ref() {
                res = insert(child.clone(), p.clone());
            } else {
                res = Arc::new(QuadNode::new_parented(quadrant[i], &node));
                res.objects.write().insert(p.clone());
                build(res.clone());
                _lock.replace(res.clone());
                node.active.fetch_or(1_u8 << i, Ordering::SeqCst);
            }
            node.objects.write().remove(&p);
            break;
        }
    }
    res
}

fn collision_detect_at(body: &Point, node: &Ptr) -> Vector2<f64> {
    let mut ans = Vector2::new(0.0, 0.0);
    for obj in node.objects.read().iter() {
        if (obj.x != body.x || obj.y != body.y) && check(body, obj) {
            let v0a = VMAP.read().get(body).unwrap().clone();
            let v0b = VMAP.read().get(obj).unwrap().clone();
            let delta_xx = body.x - obj.x;
            let delta_xy = body.y - obj.y;
            let delta_vx = v0a.x - v0b.x;
            let delta_vy = v0a.y - v0b.y;

            let coefficient = 2.0 * obj.mass / (body.mass + obj.mass)
                * (delta_xx * delta_vx + delta_xy * delta_vy)
                / (delta_xx * delta_xx + delta_xy * delta_xy);
            ans.x -= coefficient * delta_xx;
            ans.y -= coefficient * delta_xy;
        }
    }
    ans
}

fn collision_detect_down(body: &Point, node: &Ptr) -> Vector2<f64> {
    let mut ans = collision_detect_at(body, &node);

    let mut counter = 0;
    let mut atom = node.active.load(Relaxed);
    while atom > 0 {
        if atom & 1 == 1 {
            let tmp = node.children[counter].read().as_ref().cloned().unwrap();
            if tmp.region.touch(&body) {
                let res = collision_detect_down(body, &tmp);
                ans.x += res.x;
                ans.y += res.y;
            }
        }
        counter += 1;
        atom >>= 1;
    }

    ans
}


fn collision_detect_up(body: &Point, node: Ptr, now: Vector2<f64>) -> Vector2<f64> {
    if !node.region.can_touch(body) {now}
    else {
        let next = now + collision_detect_at(body, &node);
        if let Some(f) = node.parent.as_ref().and_then(|x| x.upgrade()) {
            collision_detect_up(body, f, next)
        } else {
            next
        }
    }
}

pub fn collision_detect(body: &Point, level: Ptr) -> Vector2<f64> {
    let res = collision_detect_down(body, &level);
    if let Some(f) = level.parent.as_ref().and_then(|x| x.upgrade()) {
        collision_detect_up(body, f, res)
    } else {
        res
    }
}

fn check_limit(a: &Point, b: &Ptr) -> (bool, f64, Vector2<f64>) {
    unsafe {
        let center = *b.mass_center_reader / *b.mass_reader;
        //println!("{:?}, {:?}", *b.mass_reader, b.mass.read().deref());
        let scale = (b.region.0 - b.region.1).norm_squared();
        let dist = (a.coords() - center).norm_squared();
        (scale / dist / 2.0 < DIST_SCALE_LIMIT, dist, center)
    }
}

pub(crate) fn get_impact(a: &Point, b: Ptr) -> (f64, f64) {
    if let (true, dist, center) = check_limit(a, &b) {
        unsafe {
            let alpha = G * a.mass * (*b.mass_reader) / dist / dist.sqrt();
            ((center.x - a.x) * alpha, (center.y - a.y) * alpha)
        }
    } else {
        let mut now = (0.0, 0.0);
        for obj in b.objects.read().iter() {
            if !check(a, obj) {
                let delta_x = obj.x - a.x;
                let delta_y = obj.y - a.y;
                let dist = delta_x * delta_x + delta_y * delta_y;
                let alpha = G * a.mass * obj.mass / dist / dist.sqrt();
                now.0 += delta_x * alpha;
                now.1 += delta_y * alpha;
            }
        }
        let mut counter = 0;
        let mut atom = b.active.load(Relaxed);
        while atom > 0 {
            if atom & 1 == 1 {
                let tmp = b.children[counter].read().as_ref().cloned().unwrap();
                let res = get_impact(a, tmp);
                now.0 += res.0;
                now.1 += res.1;
            }
            counter += 1;
            atom >>= 1;
        }
        now
    }
}


