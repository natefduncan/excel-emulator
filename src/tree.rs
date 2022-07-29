use petgraph::{
    stable_graph::{StableGraph, EdgeIndex, NodeIndex}
};
use thiserror::Error; 

#[derive(Error, Debug)]
pub enum TreeError {
    #[error("node `{0}` is missing")]
    NodeMissing(u32)
}

pub trait NodeTrait { }
pub trait EdgeTrait { } 
pub type NodeId = NodeIndex<u32>; 
pub type EdgeId = EdgeIndex<u32>; 
pub type Node = Box<dyn NodeTrait>; 
pub type Edge = Box<dyn EdgeTrait>; 
pub type Tree = StableGraph<Node, Edge>;  

pub fn add_node(tree: &mut Tree, node: Node) -> NodeId {
    tree.add_node(node) 
}

pub fn add_edge(tree: &mut Tree, from: NodeId, to: NodeId, edge: Edge) -> EdgeId {
    tree.add_edge(from, to, edge)
}

pub fn add_child(tree: &mut Tree, parent: NodeId, child: Node, edge: Edge) -> (NodeId, EdgeId) {
    let node_id : NodeId = add_node(tree, child); 
    (node_id, add_edge(tree, node_id, parent, edge))
}

pub fn add_parent(tree: &mut Tree, child: NodeId, parent: Node, edge: Edge) -> (NodeId, EdgeId) {
    let node_id : NodeId = add_node(tree, parent);
    (node_id, add_edge(tree, child, node_id, edge))
}

pub fn remove_node(tree: &mut Tree, node_id: NodeId) {
    tree.remove_node(node_id);
}

pub fn remove_edge(tree: &mut Tree, edge_id: EdgeId) {
    tree.remove_edge(edge_id); 
}

#[cfg(test)]
mod tests {
	use crate::tree::*; 

	struct Node(u32); 
	struct Edge(String); 
	impl NodeTrait for Node {}
	impl EdgeTrait for Edge {} 

    #[test]
	fn add_node_test() {
		let mut tree = Tree::new(); 
		let a = Box::new(Node(1)); 
		let b = Box::new(Node(2)); 
		add_node(&mut tree, a); 
		add_node(&mut tree, b);
	}

	#[test]
	fn remove_node_test() {
		let mut tree = Tree::new(); 
		let a = Box::new(Node(1)); 
		let b = Box::new(Node(2)); 
		let a_id = add_node(&mut tree, a); 
		let b_id = add_node(&mut tree, b);
		remove_node(&mut tree, a_id); 
		remove_node(&mut tree, b_id); 
	}

	#[test]
	fn parent_child_test() {
		let mut tree = Tree::new(); 
		let a = Box::new(Node(1)); 
		let b = Box::new(Node(2)); 
		let c = Box::new(Node(3)); 
		let a_id = add_node(&mut tree, a); 
		let ab_edge = Box::new(Edge(String::from("AB Edge")));
		let ac_edge = Box::new(Edge(String::from("AC Edge"))); 
		let (_b_id, _ab_id) = add_parent(&mut tree, a_id, b, ab_edge); 
		let (_c_id, _ac_id) = add_child(&mut tree, a_id, c, ac_edge); 
	}
}
