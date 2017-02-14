use std::collections::BinaryHeap;
use cgmath::{Vector3, InnerSpace};
use ideal::IdVec;
use event::{SiteEvent, CircleEvent};
use beach_line::{BeachLine, Arc};
use diagram::{Diagram, Vertex};

#[derive(Default)]
struct Builder {
    circle_events: BinaryHeap<CircleEvent>,
    site_events: Vec<SiteEvent>,
    beach: BeachLine,
    diagram: Diagram,
    starts: IdVec<Vertex>,
}

impl Builder {
    fn new(positions: &[Vector3<f64>]) -> Self {
        let mut builder = Builder::default();
        for &position in positions {
            builder.site_events.push(SiteEvent::from(position));
        }
        builder
    }
    
    fn build(mut self, relaxations: usize) -> Diagram {
        self.build_iter();
        for _ in 1..relaxations {
            self.reset();
            self.build_iter();
        }
        self.finish();
        self.diagram
    }   

    fn build_iter(&mut self) {
        self.site_events.sort();
        loop {
            match (self.diagram.cells().len() >= self.site_events.len(), self.circle_events.is_empty()) {
                (true, true) => break,
                (true, false) => self.circle_event(),
                (false, true) => self.site_event(),
                (false, false) => if self.site_events[self.diagram.cells().len()].theta.value < self.circle_events.peek().unwrap().theta {
                    self.site_event()
                } else {
                    self.circle_event()
                }
            }
        }
    }

    fn reset(&mut self) {
        self.site_events.clear();
        self.circle_events.clear();
        self.starts.clear();
        self.beach.clear();
        for cell in self.diagram.cells() {
            self.site_events.push(SiteEvent::from(self.diagram.center(cell)));
        }
        self.diagram.clear();
    }

    fn finish(&mut self) {
        for edge in self.diagram.edges() {
            let mut common = Vec::new(); 
            let (vertex0, vertex1) = self.diagram.edge_vertices(edge);  
            for &cell0 in self.diagram.vertex_cells(vertex0) {
                for &cell1 in self.diagram.vertex_cells(vertex1) {
                    if cell0 == cell1 {
                        common.push(cell0);
                    }
                }
            }
            assert_eq!(common.len(), 2);
            self.diagram.set_edge_cells(edge, common[0], common[1]);
        }
        for vertex in self.diagram.vertices() {
            assert_eq!(self.diagram.vertex_cells(vertex).len(), 3);
            assert_eq!(self.diagram.vertex_edges(vertex).len(), 3);
        }
    }

    fn site_event(&mut self) {
        //println!("site");
        let theta = self.site_events[self.diagram.cells().len()].theta.value;
        let cell = self.diagram.add_cell();
        let arc = self.beach.insert(cell, &self.site_events);
        let (prev, next) = self.beach.neighbors(arc);
        if prev != arc {
            self.create_temporary(prev, arc);
            if prev != next {
                self.attach_circle(prev, theta);
                self.attach_circle(next, theta);
            }
        }
    }

    fn circle_event(&mut self) {
        //println!("circle");
        let CircleEvent { arc, theta } = self.circle_events.pop().unwrap();
        if let Some(center) = self.beach.center(arc) {
            let (prev, next) = self.beach.neighbors(arc);
            self.beach.detach_circle(arc);
            self.beach.detach_circle(prev);
            self.beach.detach_circle(next);
            let vertex = self.diagram.add_vertex(center, [self.beach.cell(prev), self.beach.cell(arc), self.beach.cell(next)]);
            //println!("vertex");
            self.create_edge(prev, vertex);
            self.create_edge(arc, vertex);
            self.beach.remove(arc);
            if self.beach.prev(prev) == next {
                self.create_edge(next, vertex);
                self.beach.remove(prev);
                self.beach.remove(next);
            } else {
                if self.attach_circle(prev, theta) {
                    let start = self.starts.push(vertex);
                    self.beach.set_start(prev, start);
                }
                self.attach_circle(next, theta);
            }
        }
    }

    fn create_temporary(&mut self, arc0: Arc, arc1: Arc) {
        let start = self.starts.push(Vertex::invalid());
        self.beach.set_start(arc0, start);
        self.beach.set_start(arc1, start);
    }

    fn create_edge(&mut self, arc: Arc, end: Vertex) {
        let start = self.beach.start(arc);
        if start.is_valid() {
            let vertex = self.starts[start];
            if vertex.is_valid() {
                //println!("edge");
                self.diagram.add_edge(vertex, end);
            } else {
                self.starts[start] = end;
            }
        }
    }
    
    fn attach_circle(&mut self, arc: Arc, min: f64) -> bool {
        let (prev, next) = self.beach.neighbors(arc);
        let position = self.arc_position(arc);
        let from_prev = self.arc_position(prev) - position;
        let from_next = self.arc_position(next) - position;
        let center = from_prev.cross(from_next).normalize();
        //println!("{:?} {:?} {:?}", center, center.z.acos(), center.dot(position).acos());
        let theta = center.z.acos() + center.dot(position).acos();
        if theta >= min {
            //println!("{} {} -> circle", theta, min);
            self.beach.attach_circle(arc, center);
            self.circle_events.push(CircleEvent {
                theta: theta,
                arc: arc,
            });
            true
        } else {
            //println!("{} {} -> nope", theta, min);
            false
        }
    }
    
    fn arc_position(&self, arc: Arc) -> Vector3<f64> {
        self.site_events[self.beach.cell(arc).index()].position
    }
}

pub fn build(positions: &[Vector3<f64>], relaxations: usize) -> Diagram {
    Builder::new(positions).build(relaxations)
}
