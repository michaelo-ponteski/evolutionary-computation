use crate::heuristics::insertion::{grow_by_insertion, start_from};
use crate::instance::Instance;
use crate::solution::Tour;

/// Nearest neighbour, appending only at the end of the path.
///
/// "Nearest" means the smallest objective increase: `d(last, j) + c_j`.
/// The cycle is closed (last -> first) only implicitly, after construction.
pub fn nn_end(inst: &Instance, start: usize) -> Tour {
    let (mut tour, mut used) = start_from(inst, start);
    while tour.len() < inst.select_count() {
        let last = tour[tour.len() - 1];
        // min_by_key returns the first minimum, so ties go to the lowest index.
        let next = (0..inst.n)
            .filter(|&j| !used[j])
            .min_by_key(|&j| inst.d(last, j) + inst.costs[j])
            .expect("fewer unused nodes than needed");
        tour.push(next);
        used[next] = true;
    }
    tour
}

/// Nearest neighbour that may insert anywhere in the open path: at the front,
/// at the end, or between two consecutive nodes. The closing edge is ignored
/// during construction, as in the plain NN.
pub fn nn_anywhere(inst: &Instance, start: usize) -> Tour {
    let (mut tour, mut used) = start_from(inst, start);
    grow_by_insertion(inst, &mut tour, &mut used, false);
    tour
}
