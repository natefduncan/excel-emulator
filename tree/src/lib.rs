pub mod tree {
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
	pub type NodeId = NodeIndex<u32>; 
	pub type EdgeId = EdgeIndex<u32>; 
	pub type Node = Box<dyn NodeTrait>; 
    pub type Tree = StableGraph<Node, ()>;  

    pub fn add_node(tree: &mut Tree, node: Node) -> NodeId {
        tree.add_node(node) 
    }

	pub fn add_edge(tree: &mut Tree, from: NodeId, to: NodeId) -> EdgeId {
		tree.add_edge(from, to, ())
	}

	pub fn add_child(tree: &mut Tree, parent: NodeId, child: Node) -> (NodeId, EdgeId) {
		let node_id : NodeId = add_node(tree, child); 
		(node_id, add_edge(tree, node_id, parent))
	}
	
	pub fn add_parent(tree: &mut Tree, child: NodeId, parent: Node) -> (NodeId, EdgeId) {
		let node_id : NodeId = add_node(tree, parent);
		(node_id, add_edge(tree, child, node_id))
	}

	pub fn remove_node(tree: &mut Tree, node_id: NodeId) {
		tree.remove_node(node_id);
	}

	pub fn remove_edge(tree: &mut Tree, edge_id: EdgeId) {
		tree.remove_edge(edge_id); 
	}
}

#[cfg(test)]
mod tests {
	use crate::tree::*; 

	struct Node(u32); 
	impl NodeTrait for Node {}

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
}
