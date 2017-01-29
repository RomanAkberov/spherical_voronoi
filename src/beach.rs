use events::Circle;
use red_black_tree::{RedBlackTree, Node};
use diagram::{Vertex, Face};

#[derive(Copy, Clone)]
pub enum ArcStart {
    None,
    Vertex(Vertex),
    Temporary(usize),
}

pub struct ArcData {
    face: Face,
    start: ArcStart,
    circle: Option<Circle>,
}

pub type Arc = Node<ArcData>;

pub struct Beach {
    arcs: RedBlackTree<ArcData>,  
}

impl Beach {
    pub fn root(&self) -> Option<Arc> {
        self.arcs.root()
    }

    pub fn insert_after(&mut self, arc: Option<Arc>, face: Face) -> Arc {
        self.arcs.insert_after(arc, ArcData {
            face: face,
            start: ArcStart::None,
            circle: None,
        })
    }
    
    pub fn remove(&mut self, arc: Arc) {
        self.arcs.remove(arc);
    }

    pub fn circle(&self, arc: Arc) -> Option<Circle> {
        self.arcs[arc].circle    
    }
    
    pub fn set_circle(&mut self, arc: Arc, circle: Option<Circle>) {
        self.arcs[arc].circle = circle;
    }
    
    pub fn face(&self, arc: Arc) -> Face {
        self.arcs[arc].face
    }
    
    pub fn start(&mut self, arc: Arc) -> ArcStart {
        self.arcs[arc].start    
    }
    
    pub fn set_start(&mut self, arc: Arc, start: ArcStart) {
        self.arcs[arc].start = start;
    }
    
    pub fn len(&self) -> usize {
        self.arcs.len()
    }

    pub fn next(&self, arc: Arc) -> Arc {
        self.arcs.next(arc).unwrap_or_else(|| self.arcs.first(self.arcs.root().unwrap()))
    }

    pub fn prev(&self, arc: Arc) -> Arc {
        self.arcs.prev(arc).unwrap_or_else(|| self.arcs.last(self.arcs.root().unwrap()))
    }

    pub fn left(&self, arc: Arc) -> Option<Arc> {
        self.arcs.left(arc)
    }

    pub fn right(&self, arc: Arc) -> Option<Arc> {
        self.arcs.right(arc)
    }

    pub fn first(&self) -> Arc {
        self.arcs.first(self.root().unwrap())
    }

    pub fn last(&self) -> Arc {
        self.arcs.last(self.root().unwrap())
    }
}

impl Default for Beach {
    fn default() -> Self {
        Beach {
            arcs: Default::default(),
        }
    }
}