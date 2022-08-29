use crate::tree::{
    NodeTrait, NodeId, Tree, 
    add_node, add_edge, add_child, add_parent, remove_node, remove_edge
}; 
use std::collections::HashMap; 

#[derive(Hash, PartialEq, Eq)]
pub struct CellId {
    sheet_id: usize, 
    row_id: usize,
    column_id: usize
}
impl NodeTrait for CellId {}
pub struct DependencyTree {
    tree: Tree, 
    nodes: HashMap<CellId, NodeId>
}

/*
Precedent cells — cells that are referred to by a formula in another cell. For example, if cell D10 contains the formula =B5, then cell B5 is a precedent to cell D10.

Dependent cells — these cells contain formulas that refer to other cells. For example, if cell D10 contains the formula =B5, cell D10 is a dependent of cell B5.
*/

impl DependencyTree {
    pub fn add_precedent(&mut self, precedent: CellId, cell: CellId) {
        let parent: &NodeId = self.nodes.get(&cell).unwrap(); 
        add_child(&mut self.tree, *parent, Box::new(precedent)); 
    } 
}
