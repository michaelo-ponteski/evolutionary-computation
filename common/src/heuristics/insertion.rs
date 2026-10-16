use crate::instance::Instance;
use crate::solution::Tour;

/// Every place where node `j` can be inserted into `tour`, as
/// `(objective delta, index)`. Inserting at index `p` puts `j` right before
/// `tour[p]` (`p == len` = append).
///
/// `closed == false` treats `tour` as an open path: `j` may go at the front, at
/// the end, or between consecutive nodes, and the closing edge does not exist yet.
/// `closed == true` treats `tour` as a cycle: `j` goes into one of its edges,
/// including last -> first.
///
/// The delta always includes the node cost `c_j`, so it is the exact change of
/// the objective, not just the change of length.
pub fn insertion_options(
    inst: &Instance,
    tour: &[usize],
    j: usize,
    closed: bool,
) -> impl Iterator<Item = (i32, usize)> {
    let m = tour.len();
    let c = inst.costs[j];

    // Edges (tour[i], tour[i + 1]); in a cycle also the closing edge.
    let edges = if closed { m } else { m - 1 };
    let inner = (0..edges).map(move |i| {
        let (a, b) = (tour[i], tour[(i + 1) % m]);
        (inst.d(a, j) + inst.d(j, b) - inst.d(a, b) + c, i + 1)
    });
    let ends = (!closed)
        .then(|| [(inst.d(j, tour[0]) + c, 0), (inst.d(tour[m - 1], j) + c, m)])
        .into_iter()
        .flatten();
    inner.chain(ends)
}

/// Cheapest place to insert node `j` into `tour`, as `(objective delta, index)`.
/// See [`insertion_options`] for the meaning of `closed` and of the index.
pub fn cheapest_insertion(inst: &Instance, tour: &[usize], j: usize, closed: bool) -> (i32, usize) {
    let mut best = (i32::MAX, 0);
    for (delta, pos) in insertion_options(inst, tour, j, closed) {
        // Strict `<`: on ties the earlier option stays the best.
        if delta < best.0 {
            best = (delta, pos);
        }
    }
    best
}

/// The cheapest insertion of `j` (`(delta, index)`) and the delta of the
/// second cheapest one, or `None` if `j` has only one place to go.
pub fn two_cheapest_insertions(
    inst: &Instance,
    tour: &[usize],
    j: usize,
    closed: bool,
) -> ((i32, usize), Option<i32>) {
    let mut best = (i32::MAX, 0);
    let mut second = i32::MAX;
    for (delta, pos) in insertion_options(inst, tour, j, closed) {
        // Strict `<`: on ties the earlier option stays the best.
        if delta < best.0 {
            second = best.0;
            best = (delta, pos);
        } else if delta < second {
            second = delta;
        }
    }
    (best, (second != i32::MAX).then_some(second))
}

/// Greedily grows `tour` until it has `select_count` nodes: in every step it
/// inserts the (node, position) pair with the smallest objective increase.
pub fn grow_by_insertion(inst: &Instance, tour: &mut Tour, used: &mut [bool], closed: bool) {
    while tour.len() < inst.select_count() {
        let mut best = (i32::MAX, 0, 0); // (delta, node, position)
        for j in (0..inst.n).filter(|&j| !used[j]) {
            let (delta, pos) = cheapest_insertion(inst, tour, j, closed);
            if delta < best.0 {
                best = (delta, j, pos);
            }
        }
        let (_, j, pos) = best;
        tour.insert(pos, j);
        used[j] = true;
    }
}

/// Starts a 2-node cycle `s -> j -> s` from `start` and its "nearest" node,
/// i.e. the one minimising `d(s, j) + c_j` (the Lab 1 greedy cycle).
pub(crate) fn start_cycle(inst: &Instance, start: usize) -> (Tour, Vec<bool>) {
    start_cycle_by(inst, start, |j| inst.d(start, j) + inst.costs[j])
}

/// Like [`start_cycle`], but picks the node with the smallest exact objective
/// increase `2 d(s, j) + c_j`: the cycle `s -> j -> s` uses the edge twice.
pub(crate) fn start_cycle_exact(inst: &Instance, start: usize) -> (Tour, Vec<bool>) {
    start_cycle_by(inst, start, |j| 2 * inst.d(start, j) + inst.costs[j])
}

/// 2-node cycle from `start` and the unused node with the smallest `key`.
fn start_cycle_by(inst: &Instance, start: usize, key: impl Fn(usize) -> i32) -> (Tour, Vec<bool>) {
    let (mut tour, mut used) = start_from(inst, start);
    // min_by_key returns the first minimum, so ties go to the lowest index.
    let nearest = (0..inst.n)
        .filter(|&j| !used[j])
        .min_by_key(|&j| key(j))
        .expect("instance has a single node");
    tour.push(nearest);
    used[nearest] = true;
    (tour, used)
}

/// Starts a solution from a single node.
pub(crate) fn start_from(inst: &Instance, start: usize) -> (Tour, Vec<bool>) {
    let mut tour = Vec::with_capacity(inst.select_count());
    let mut used = vec![false; inst.n];
    tour.push(start);
    used[start] = true;
    (tour, used)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solution::objective;
    use rand::rngs::StdRng;
    use rand::seq::SliceRandom;
    use rand::{RngExt, SeedableRng};

    /// Objective of an open path: no closing edge.
    fn path_objective(inst: &Instance, tour: &[usize]) -> i32 {
        let len: i32 = tour.windows(2).map(|w| inst.d(w[0], w[1])).sum();
        len + tour.iter().map(|&v| inst.costs[v]).sum::<i32>()
    }

    /// The reported delta must equal the real objective change, and no
    /// position may be cheaper than the one reported.
    #[test]
    fn delta_matches_objective_change() {
        let inst = Instance::load("TSPA.csv").unwrap();
        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..50 {
            let mut nodes: Vec<usize> = (0..inst.n).collect();
            nodes.shuffle(&mut rng);
            let (tour, rest) = nodes.split_at(1 + rng.random_range(0..20));
            let j = rest[0];

            for closed in [false, true] {
                let eval = |t: &[usize]| {
                    if closed {
                        objective(&inst, t)
                    } else {
                        path_objective(&inst, t)
                    }
                };
                let (delta, pos) = cheapest_insertion(&inst, tour, j, closed);
                // In a cycle, prepending is the same as appending, so skip index 0.
                let positions = if closed {
                    1..=tour.len()
                } else {
                    0..=tour.len()
                };
                let mut best_real = i32::MAX;
                for p in positions {
                    let mut t = tour.to_vec();
                    t.insert(p, j);
                    let real = eval(&t) - eval(tour);
                    if p == pos {
                        assert_eq!(delta, real, "closed={closed} pos={p}");
                    }
                    best_real = best_real.min(real);
                }
                assert_eq!(
                    delta, best_real,
                    "closed={closed}: a cheaper position exists"
                );

                let mut deltas: Vec<i32> =
                    insertion_options(&inst, tour, j, closed).map(|(d, _)| d).collect();
                deltas.sort_unstable();
                let (_, second) = two_cheapest_insertions(&inst, tour, j, closed);
                assert_eq!(second, deltas.get(1).copied(), "closed={closed}: second best");
            }
        }
    }
}
