// Trying to build a rust based version of micrograd deep learning framework
use std::rc::Rc;
use std::cell::RefCell;
use Config::EdgeNoLabel;
use petgraph::data::{DataMap, FromElements};
use petgraph::dot::{Config, Dot};
use petgraph::Graph;
use petgraph::graph::{DiGraph, NodeIndex};


#[derive(Debug, Clone, PartialEq)]
enum Op {
    Add, Mul,None
}

#[derive(Debug, Clone, PartialEq)]
struct Value {
    pub data: f32,
    pub grad: f32,
    pub prev: Vec<Rc<RefCell<Value>>>,
    pub label: Option<String>,
    pub op: Op,
}

impl Value {

    fn backward(&mut self) {
        match self.op {
            Op::Add => {
                self.prev[0].borrow_mut().grad += self.grad;
                self.prev[1].borrow_mut().grad += self.grad;
            }
            Op::Mul => {
                self.prev[0].borrow_mut().grad += self.grad * self.prev[1].borrow().data;
                self.prev[1].borrow_mut().grad += self.grad * self.prev[0].borrow().data;
            }
            _ => {}
        }
    }
}

fn add(lhs: Rc<RefCell<Value>>, rhs: Rc<RefCell<Value>>) -> Rc<RefCell<Value>> {
    let data = lhs.clone().borrow().data.clone() + rhs.clone().borrow().data.clone();
    Rc::new(RefCell::new(Value {
        data,
        grad: 0.0,
        prev: vec![lhs, rhs],
        op: Op::Add,
        label: None
    }))
}

fn mul(lhs: Rc<RefCell<Value>>, rhs: Rc<RefCell<Value>>) -> Rc<RefCell<Value>> {
    let data = lhs.clone().borrow().data.clone() * rhs.clone().borrow().data.clone();
    Rc::new(RefCell::new(Value {
        data,
        grad: 0.0,
        prev: vec![lhs, rhs],
        op: Op::Mul,
        label: None
    }))
}

fn build_nodes_and_edges(root: Rc<RefCell<Value>>) -> DiGraph<String, ()>{
    let mut d = DiGraph::<String, (), >::new();
    build_graph(root, &mut d);
    d
}

fn build_graph(node: Rc<RefCell<Value>>, d: &mut DiGraph<String, ()>) -> Option<NodeIndex> {
    let node_label = format!("{} | data={}",
                             node.borrow().clone().label
                                 .unwrap_or("--".to_string()).clone(),
                             node.borrow().clone().data);
    if d.node_weights().all(|w| *w != node_label) {
        let nix = d.add_node(node_label.clone());
        let op_node = match node.borrow().op {
            Op::Add => {d.add_node("+".to_string())}
            Op::Mul => {d.add_node("*".to_string())}
            Op::None => {
                return Some(nix)
            }
        };
        d.add_edge(op_node, nix, ());
        for child in node.borrow().clone().prev {
            let idx= build_graph(child, d);
            if idx.is_some() {
                d.add_edge(idx.unwrap(), op_node, ());
            }
        }
        return Some(nix)
    }
    None
}

fn main() {
    let a = Rc::new(RefCell::new(Value {
        data: 1.1,
        grad: 0.0,
        prev: vec![],
        op: Op::None,
        label: Some("a".to_string())
    }));
    let b = Rc::new(RefCell::new(Value {
        data: 2.0,
        grad: 0.0,
        prev: vec![],
        op: Op::None,
        label: Some("b".to_string())
    }));
    let c = add(a.clone(), b.clone());
    c.borrow_mut().label = Some("c".to_string());
    let d = mul(a.clone(), b.clone());
    d.borrow_mut().label = Some("d".to_string());
    let e = mul(c.clone(), d.clone());
    e.borrow_mut().label = Some("e".to_string());
    let di = build_nodes_and_edges(e);
    println!("{:?}", Dot::with_config(&di, &[EdgeNoLabel]));
}