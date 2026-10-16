use crate::heuristics::insertion::{start_cycle_exact, start_from, two_cheapest_insertions};
use crate::instance::Instance;
use crate::solution::Tour;

/// Weight of the 2-regret in the selection criterion
/// `score = lambda * regret - (1 - lambda) * best_delta` (maximised).
///
/// `REGRET` is the pure greedy 2-regret, `WEIGHTED` gives both parts equal
/// weights. `lambda == 0` is the plain greedy rule (smallest objective increase).
pub const REGRET: f64 = 1.0;
pub const WEIGHTED: f64 = 0.5;

/// Greedily grows `tour` until it has `select_count` nodes. In every step each
/// unused node is scored by its two cheapest insertions (see
/// [`two_cheapest_insertions`] for `closed`); the node with the highest score
/// goes into its cheapest place.
///
/// The 2-regret of a node is the second cheapest delta minus the cheapest one:
/// how much we lose if this node's best place is taken before we insert it.
pub fn grow_by_regret(inst: &Instance, tour: &mut Tour, used: &mut [bool], closed: bool, lambda: f64) {
    while tour.len() < inst.select_count() {
        let mut best = (f64::NEG_INFINITY, 0, 0); // (score, node, position)
        for j in (0..inst.n).filter(|&j| !used[j]) {
            let ((delta, pos), second) = two_cheapest_insertions(inst, tour, j, closed);
            // A single place to go has nothing to regret.
            let regret = second.map_or(0, |s| s - delta);
            let score = lambda * regret as f64 - (1.0 - lambda) * delta as f64;
            // Strict `>`: ties go to the lowest node index.
            if score > best.0 {
                best = (score, j, pos);
            }
        }
        let (_, j, pos) = best;
        tour.insert(pos, j);
        used[j] = true;
    }
}

/// Extended nearest neighbour (insertion anywhere in the open path) with the
/// regret criterion. The closing edge is ignored during construction.
pub fn regret_nn(inst: &Instance, start: usize, lambda: f64) -> Tour {
    let (mut tour, mut used) = start_from(inst, start);
    grow_by_regret(inst, &mut tour, &mut used, false, lambda);
    tour
}

/// Greedy cycle with the regret criterion: starts from `start` and the node
/// with the smallest objective of the cycle `s -> j -> s` (`2 d(s, j) + c_j`),
/// then inserts into the edges of the cycle.
pub fn regret_cycle(inst: &Instance, start: usize, lambda: f64) -> Tour {
    let (mut tour, mut used) = start_cycle_exact(inst, start);
    grow_by_regret(inst, &mut tour, &mut used, true, lambda);
    tour
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heuristics::insertion::grow_by_insertion;
    use crate::heuristics::nearest_neighbor::nn_anywhere;

    /// With no weight on the regret the criterion is the plain greedy rule:
    /// the NN variant must reproduce the Lab 1 heuristic exactly, and the cycle
    /// variant greedy insertion from the same (exact) 2-node start.
    #[test]
    fn zero_lambda_is_greedy() {
        for file in ["TSPA.csv", "TSPB.csv"] {
            let inst = Instance::load(file).unwrap();
            for s in (0..inst.n).step_by(7) {
                assert_eq!(regret_nn(&inst, s, 0.0), nn_anywhere(&inst, s), "{file} start {s}");

                let (mut greedy, mut used) = start_cycle_exact(&inst, s);
                grow_by_insertion(&inst, &mut greedy, &mut used, true);
                assert_eq!(regret_cycle(&inst, s, 0.0), greedy, "{file} start {s}");
            }
        }
    }
}
