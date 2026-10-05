use rand::Rng;
use rand::seq::SliceRandom;

use crate::instance::Instance;
use crate::solution::Tour;

/// Uniformly random subset of `select_count` nodes in uniformly random order.
pub fn random_solution(inst: &Instance, rng: &mut impl Rng) -> Tour {
    let mut nodes: Tour = (0..inst.n).collect();
    // Shuffling all n and keeping the first k gives both a random subset and a
    // random order in one pass.
    nodes.shuffle(rng);
    nodes.truncate(inst.select_count());
    nodes
}
