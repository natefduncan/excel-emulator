use petgraph::{
    algo::toposort, 
    graphmap::DiGraphMap, 
    dot::{Config, Dot}, 
    prelude::Dfs,
};

use crate::cell::Cell; 
use std::fmt; 

pub struct DependencyTree {
    tree: DiGraphMap<Cell, u8>
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

    pub fn add_cell(&mut self, cell: Cell) {
        self.tree.add_node(cell); 
    }

    pub fn cell_exists(&self, cell: &Cell) -> bool {
        self.tree.contains_node(*cell)
    }

    pub fn add_cell_if_missing(&mut self, cell: &Cell) {
        if self.tree.contains_node(*cell) {
            self.add_cell(*cell); 
        }
    }

    pub fn add_precedent(&mut self, precedent: &Cell, cell: &Cell) {
        self.add_cell_if_missing(precedent);
        self.add_cell_if_missing(cell);
        if !self.tree.contains_edge(*cell, *precedent) {
            self.tree.add_edge(*precedent, *cell, 0); 
        }
   } 

    pub fn is_precedent_of(&self, cell1: &Cell, cell2: &Cell) -> bool {
        self.tree.contains_edge(*cell1, *cell2)
    }

    pub fn is_dependent_of(&self, cell1: &Cell, cell2: &Cell) -> bool {
        self.tree.contains_edge(*cell2, *cell1) 
    } 

    pub fn get_order(&self) -> Vec<Cell> {
        match toposort(&self.tree, None) {
            Ok(order) => {
                order
                // order.into_iter().rev().collect::<Vec<Cell>>()
            }, 
            Err(e) => panic!("{:?}", e) 
        } 
    } 

    pub fn mark_for_recalculation(&mut self, root: &Cell) {
        let mut dfs = Dfs::new(&self.tree, root.clone());
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
