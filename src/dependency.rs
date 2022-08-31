use petgraph::{
    graphmap::DiGraphMap, 
    algo::toposort, 
    dot::{Dot, Config}
}; 
use std::{fmt, cmp::Ordering}; 

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct CellId {
    sheet: usize, 
    row: usize,
    column: usize
}

impl PartialOrd for CellId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CellId {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.sheet == other.sheet {
            if self.row != other.row {
                self.row.cmp(&other.row)
            } else {
                self.column.cmp(&other.column)
            }
        } else {
            self.sheet.cmp(&other.sheet)
        }
    }
}

impl From<(usize, usize, usize)> for CellId {
    fn from((sheet, row, column) : (usize, usize, usize)) -> CellId {
        CellId { sheet, row, column }
    }
}

impl fmt::Display for CellId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.sheet, self.row, self.column)
    }
}

pub struct DependencyTree {
    tree: DiGraphMap<CellId, u8>, 
}

/*
Precedent cells — cells that are referred to by a formula in another cell. For example, if cell D10 contains the formula =B5, then cell B5 is a precedent to cell D10.

Dependent cells — these cells contain formulas that refer to other cells. For example, if cell D10 contains the formula =B5, cell D10 is a dependent of cell B5.
*/

impl DependencyTree {
    pub fn new() -> DependencyTree {
        DependencyTree { tree: DiGraphMap::new() }
    }

    pub fn add_cell(&mut self, cell: CellId) {
        self.tree.add_node(cell); 
    }

    pub fn add_cell_if_missing(&mut self, cell: &CellId) {
        if self.tree.contains_node(*cell) {
            self.add_cell(*cell); 
        }
    }

    pub fn add_precedent(&mut self, precedent: &CellId, cell: &CellId) {
        self.add_cell_if_missing(precedent);
        self.add_cell_if_missing(cell);
        if !self.tree.contains_edge(*precedent, *cell) {
            self.tree.add_edge(*precedent, *cell, 0); 
        }
   } 

    pub fn is_precedent_of(&self, cell1: &CellId, cell2: &CellId) -> bool {
        self.tree.contains_edge(*cell1, *cell2)
    }

    pub fn is_dependent_of(&self, cell1: &CellId, cell2: &CellId) -> bool {
        self.tree.contains_edge(*cell2, *cell1) 
    } 

    pub fn get_order(&self) -> Vec<CellId> {
        match toposort(&self.tree, None) {
            Ok(order) => {
                order.into_iter().rev().collect::<Vec<CellId>>()
            }, 
            Err(e) => panic!("{:?}", e) 
        } 

    } 
}

impl fmt::Display for DependencyTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Dot::with_config(&self.tree, &[Config::EdgeNoLabel]))
    }
}

#[cfg(test)]
mod tests {
    use crate::dependency::*; 

    #[test]
    fn test_precedent() {
        let mut tree = DependencyTree::new(); 
        let a = CellId::from((0,0,0)); 
        let b = CellId::from((1,0,0)); 
        let c = CellId::from((2,0,0)); 
        tree.add_precedent(&a, &b); // A must calculate before B 
        tree.add_precedent(&c, &b); // C must calculate before B 
        assert!(tree.is_dependent_of(&b, &a)); 
        assert_eq!(tree.is_dependent_of(&a, &b), false); 
    }

    #[test]
    fn test_order() {
        let mut tree = DependencyTree::new(); 
        let a = CellId::from((0,0,0)); 
        let b = CellId::from((1,0,0)); 
        let c = CellId::from((2,0,0)); 
        tree.add_precedent(&a, &b); // A must calculate before B 
        tree.add_precedent(&b, &c); // B must calculate before C 
        let mut order: Vec<CellId> = tree.get_order(); 
        assert_eq!(order.pop().unwrap(), a);
        assert_eq!(order.pop().unwrap(), b);
        assert_eq!(order.pop().unwrap(), c);
    }
}

