use std::f64::consts::{PI, FRAC_1_PI};
use cgmath::{Vector3, Zero};
use ideal::IdVec;
use event::SiteEvent;
use builder::{Cell, Vertex};

const HEIGHT: usize = 5;

#[derive(Debug)]
struct ArcData {
    cell: Cell,
    circle_theta: f64,
    circle_center: Vector3<f64>,
    last_theta: f64,
    last_intersection: f64,
    prev: Arc,
    next: Arc,
    prev_skips: [Arc; HEIGHT],
    next_skips: [Arc; HEIGHT],
    start: Start,
}

create_id!(Arc);
create_id!(Start);

#[derive(Debug, Default)]
pub struct BeachLine {
    arcs: IdVec<Arc, ArcData>,
    free: Vec<Arc>,
    head: Arc,
    len: usize,
    levels: [usize; HEIGHT],
    starts: IdVec<Start, Vertex>,
}

impl BeachLine {
    pub fn insert(&mut self, cell: Cell, sites: &IdVec<Cell, SiteEvent>) -> Arc {
        let arc = self.create_arc(cell);
        if self.len > 1 {
            let mut current = self.head;
            let mut level = HEIGHT - 1;
            let mut skips = [Arc::default(); HEIGHT];
            let site = &sites[cell];
            loop {
                let next_skip = self.next_skip(current, level);
                let start = self.intersect_with_next(current, site, sites);
                let end = self.intersect_with_next(next_skip, site, sites);
                if start < end {
                    current = next_skip;
                } else {
                    skips[level] = current;
                    if level > 0 {
                        level -= 1;
                    } else {
                        break;
                    }
                }
            }
            let mut next = self.next(current);
            let mut start = self.intersect_with_next(current, site, sites);
            let mut end = self.intersect_with_next(next, site, sites);
            while start < end {
                next = self.next(next);
                start = end;
                end = self.intersect_with_next(next, site, sites);
            }
            current = next;
            let current_cell = self.cell(current);
            let twin = self.create_arc(current_cell);
            let prev = self.prev(current);
            self.add_links(twin, prev, current, &mut skips);
            self.add_links(arc, twin, current, &mut skips);
        } else {
            if self.len == 0 {
                self.head = arc;
            }
            let head = self.head;
            self.add_links(arc, head, head, &mut [head; HEIGHT]);
        }
        arc
    }

    pub fn edge(&mut self, arc: Arc, end: Vertex) -> Option<Vertex> {
        let start = self.arcs[arc].start;
        if start.is_invalid() {
            return None;
        }
        let vertex = self.starts[start];
        if vertex.is_invalid() {
            self.starts[start] = end;
            None
        } else {
            Some(vertex)
        }
    }

    pub fn start(&mut self, arc: Arc, vertex: Vertex) {
        self.arcs[arc].start = self.starts.push(vertex);
    }

    pub fn neighbors(&self, arc: Arc) -> (Arc, Arc) {
        let data = &self.arcs[arc];
        (data.prev, data.next)
    }

    pub fn remove(&mut self, arc: Arc) {
        let head = self.head;
        if arc == head {
            let next_skip = self.next_skip(self.head, HEIGHT - 1);
            if next_skip != self.head {
                self.head = next_skip;
            } else {
                // promote next to HEIGHT
                let next = self.next(self.head);
                let height = self.height(next);
                self.levels[height - 1] -= 1;
                self.levels[HEIGHT - 1] += 1;
                for level in height..HEIGHT {
                    let next_skip = self.next_skip(self.head, level);
                    self.set_prev_skip(next_skip, level, next);
                    self.set_next_skip(next, level, next_skip);
                    self.set_prev_skip(next, level, head);
                    self.set_next_skip(head, level, next);
                }
                self.head = next;
            }
        }
        self.remove_links(arc);
        self.free.push(arc);
    }

    pub fn add_common_start(&mut self, arc0: Arc, arc1: Arc) {
        if arc0 != arc1 {
            let start = self.starts.push(Vertex::invalid());
            self.arcs[arc0].start = start;
            self.arcs[arc1].start = start;
        } else {
            self.arcs[arc1].start = Start::invalid();
        }
    }
    
    pub fn cell(&self, arc: Arc) -> Cell {
        self.arcs[arc].cell
    }

    pub fn circle_theta(&self, arc: Arc) -> f64 {
        self.arcs[arc].circle_theta
    }

    pub fn circle_center(&self, arc: Arc) -> Vector3<f64> {
        self.arcs[arc].circle_center
    }

    pub fn attach_circle(&mut self, arc: Arc, theta: f64, center: Vector3<f64>) {
        self.arcs[arc].circle_center = center;
        self.arcs[arc].circle_theta = theta;
    }

    pub fn detach_circle(&mut self, arc: Arc) {
        self.arcs[arc].circle_theta = ::std::f64::MIN;
    }

    pub fn prev(&self, arc: Arc) -> Arc {
        self.arcs[arc].prev
    }

    pub fn next(&self, arc: Arc) -> Arc {
        self.arcs[arc].next
    }

    fn create_arc(&mut self, cell: Cell) -> Arc {
        let data = ArcData {
            cell: cell,
            circle_center: Vector3::zero(),
            circle_theta: ::std::f64::MIN,
            last_theta: -1.0,
            last_intersection: 0.0,
            prev: Arc::default(),
            next: Arc::default(),
            prev_skips: [Arc::default(); HEIGHT],
            next_skips: [Arc::default(); HEIGHT],
            start: Start::default(),
        };
        if let Some(arc) = self.free.pop() {
            self.arcs[arc] = data;
            arc
        } else {
            self.arcs.push(data)
        }
    }

    fn skips(&self, arc: Arc, level: usize) -> (Arc, Arc) {
        let data = &self.arcs[arc];
        (data.prev_skips[level], data.next_skips[level])
    }

    fn set_prev_skip(&mut self, arc: Arc, level: usize, prev: Arc) {
        self.arcs[arc].prev_skips[level] = prev;
    }

    fn next_skip(&self, arc: Arc, level: usize) -> Arc {
        self.arcs[arc].next_skips[level]
    }

    fn set_next_skip(&mut self, arc: Arc, level: usize, next: Arc) {
        self.arcs[arc].next_skips[level] = next;
    }

    fn intersect_with_next(&mut self, arc: Arc, site: &SiteEvent, sites: &IdVec<Cell, SiteEvent>) -> f64 {
        let arc_point = &sites[self.cell(arc)];
        let next_point = &sites[self.cell(self.next(arc))];
        let data = &mut self.arcs[arc];
        if data.last_theta < site.theta.value {
            data.last_theta = site.theta.value;
            data.last_intersection = BeachLine::intersect(arc_point, next_point, site);
        }
        data.last_intersection
    }

    fn intersect(site0: &SiteEvent, site1: &SiteEvent, site2: &SiteEvent) -> f64 {
        let u1 = (site2.theta.cos - site1.theta.cos) * site0.theta.sin;
        let u2 = (site2.theta.cos - site0.theta.cos) * site1.theta.sin;
        let a = u1 * site0.phi.cos - u2 * site1.phi.cos;
        let b = u1 * site0.phi.sin - u2 * site1.phi.sin;
        let c = (site0.theta.cos - site1.theta.cos) * site2.theta.sin;
        let length = (a * a + b * b).sqrt();
        let gamma = a.atan2(b);
        let phi_plus_gamma = (c / length).asin();
        BeachLine::wrap(phi_plus_gamma - gamma - site2.phi.value)
    }

    fn wrap(mut phi: f64) -> f64 {
        phi *= 0.5 * FRAC_1_PI;
        phi -= phi.floor();
        phi * 2.0 * PI
    }

    fn add_links(&mut self, arc: Arc, prev: Arc, next: Arc, skips: &mut [Arc; HEIGHT]) {
        self.arcs[arc].prev = prev;
        self.arcs[arc].next = next;
        self.arcs[prev].next = arc;
        self.arcs[next].prev = arc;
        let height = self.insertion_height();
        for level in 0..height {
            let prev = skips[level];
            let mut next = self.next_skip(prev, level);
            if next.is_invalid() {
                next = prev;
            }
            self.set_prev_skip(arc, level, prev);
            self.set_next_skip(arc, level, next);
            self.set_prev_skip(next, level, arc);
            self.set_next_skip(prev, level, arc);
            skips[level] = arc;
        }
        self.len += 1;
        self.levels[height - 1] += 1;
    }

    fn remove_links(&mut self, arc: Arc) {
        let (prev, next) = self.neighbors(arc);
        self.arcs[prev].next = next;
        self.arcs[next].prev = prev;
        let height = self.height(arc);
        for level in 0..height {
            let (prev_skip, next_skip) = self.skips(arc, level);
            self.set_prev_skip(next_skip, level, prev_skip);
            self.set_next_skip(prev_skip, level, next_skip);
        }
        self.len -= 1;
        self.levels[height - 1] -= 1;
    }

    fn height(&self, arc: Arc) -> usize {
        for level in 0..HEIGHT {
            if self.next_skip(arc, level).is_invalid() {
                return level;
            }
        }
        HEIGHT
    }

    fn insertion_height(&self) -> usize {
        if self.len == 0 {
            return HEIGHT;
        }
        let mut best_height = 1;
        let mut best_ratio = self.levels[0];
        let mut multiplier = 1;
        for level in 0..HEIGHT {
            let ratio = self.levels[level] * multiplier;
            if ratio < best_ratio {
                best_ratio = ratio;
                best_height = level + 1;
            }
            multiplier *= 2;
        }
        best_height
    }
}

