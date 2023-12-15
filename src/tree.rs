use petgraph::{
    algo::toposort, 
    graphmap::DiGraphMap, 
    dot::{Config, Dot}, 
    prelude::Dfs,
};

use crate::cell::CellIndex; 
use std::fmt; 

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct CellNode {
    pub cell_index: CellIndex, 
    pub sheet_id: usize, 
    pub dirty: bool
}

impl fmt::Display for CellNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.cell_index)
    }
}

pub struct DependencyTree {
    tree: DiGraphMap<CellNode, u8>
}

/*
Precedent cells — cells that are referred to by a formula in another cell. For example, if cell D10 contains the formula =B5, then cell B5 is a precedent to cell D10.

Dependent cells — these cells contain formulas that refer to other cells. For example, if cell D10 contains the formula =B5, cell D10 is a dependent of cell B5.
*/
impl Default for DependencyTree {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyTree {
    pub fn new() -> DependencyTree {
        DependencyTree { tree: DiGraphMap::new() }
    }

    pub fn add_cell(&mut self, cell_node: CellNode) {
        self.tree.add_node(cell_node); 
    }

    pub fn cell_exists(&self, cell_node: &CellNode) -> bool {
        self.tree.contains_node(*cell_node)
    }

    pub fn add_cell_if_missing(&mut self, cell_node: &CellNode) {
        if self.tree.contains_node(*cell_node) {
            self.add_cell(*cell_node); 
        }
    }

    pub fn add_precedent(&mut self, precedent: &CellNode, cell_node: &CellNode) {
        self.add_cell_if_missing(precedent);
        self.add_cell_if_missing(cell_node);
        if !self.tree.contains_edge(*cell_node, *precedent) {
            self.tree.add_edge(*precedent, *cell_node, 0); 
        }
   } 

    pub fn is_precedent_of(&self, cell_node1: &CellNode, cell_node2: &CellNode) -> bool {
        self.tree.contains_edge(*cell_node1, *cell_node2)
    }

    pub fn is_dependent_of(&self, cell_node1: &CellNode, cell_node2: &CellNode) -> bool {
        self.tree.contains_edge(*cell_node2, *cell_node1) 
    } 

    pub fn get_order(&self) -> Vec<CellNode> {
        match toposort(&self.tree, None) {
            Ok(order) => {
                order
                // order.into_iter().rev().collect::<Vec<CellNode>>()
            }, 
            Err(e) => panic!("{:?}", e) 
        } 
    } 

    pub fn mark_for_recalculation(&mut self, root_node: &CellNode) {
        let mut dfs = Dfs::new(&self.tree, root_node.clone());
        while let Some(mut node_id) = dfs.next(&self.tree) {
            node_id.dirty = true; 
        }
    }
}

impl fmt::Display for DependencyTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Dot::with_config(&self.tree, &[Config::EdgeNoLabel]))
    }
}
