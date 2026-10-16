use crate::heuristics::insertion::{grow_by_insertion, start_cycle};
use crate::instance::Instance;
use crate::solution::Tour;

/// Greedy cycle (lecture version, adapted to node costs).
///
/// Start node -> add its "nearest" node, i.e. the one minimising
/// `d(s, j) + c_j`, forming the 2-node cycle s -> j -> s -> repeatedly insert
/// the node with the smallest objective increase into the best edge of the
/// current cycle (closing edge included).
pub fn greedy_cycle(inst: &Instance, start: usize) -> Tour {
    let (mut tour, mut used) = start_cycle(inst, start);
    grow_by_insertion(inst, &mut tour, &mut used, true);
    tour
}
