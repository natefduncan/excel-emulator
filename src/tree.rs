use petgraph::{
    algo::toposort, 
    graphmap::DiGraphMap, 
    dot::{Config, Dot}, 
    prelude::Dfs,
};

use crate::cell::CellIndex; 
use std::fmt; 

pub struct DependencyTree {
    tree: DiGraphMap<CellIndex, u8>
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

    pub fn add_cell(&mut self, cell_index: CellIndex) {
        self.tree.add_node(cell_index); 
    }

    pub fn cell_exists(&self, cell_index: &CellIndex) -> bool {
        self.tree.contains_node(*cell_index)
    }

    pub fn add_cell_if_missing(&mut self, cell_index: &CellIndex) {
        if self.tree.contains_node(*cell_index) {
            self.add_cell(*cell_index); 
        }
    }

    pub fn add_precedent(&mut self, precedent: &CellIndex, cell_index: &CellIndex) {
        self.add_cell_if_missing(precedent);
        self.add_cell_if_missing(cell_index);
        if !self.tree.contains_edge(*cell_index, *precedent) {
            self.tree.add_edge(*precedent, *cell_index, 0); 
        }
   } 

    pub fn is_precedent_of(&self, cell_index1: &CellIndex, cell_index2: &CellIndex) -> bool {
        self.tree.contains_edge(*cell_index1, *cell_index2)
    }

    pub fn is_dependent_of(&self, cell_index1: &CellIndex, cell_index2: &CellIndex) -> bool {
        self.tree.contains_edge(*cell_index2, *cell_index1) 
    } 

    pub fn get_order(&self) -> Vec<CellIndex> {
        match toposort(&self.tree, None) {
            Ok(order) => {
                order
                // order.into_iter().rev().collect::<Vec<CellIndex>>()
            }, 
            Err(e) => panic!("{:?}", e) 
        } 
    } 

    //pub fn mark_for_recalculation(&mut self, root: &CellIndex) {
        //let mut dfs = Dfs::new(&self.tree, root.clone());
        //while let Some(mut node_id) = dfs.next(&self.tree) {
            //node_id.dirty = true; 
        //}
    //}
}

impl fmt::Display for DependencyTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Dot::with_config(&self.tree, &[Config::EdgeNoLabel]))
    }
}
