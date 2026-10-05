use crate::instance::Instance;
use crate::solution::{Tour, objective, validate};

pub struct Stats {
    pub min: i32,
    pub max: i32,
    pub avg: f64,
    pub best: Tour,
}

/// Runs `method(run_index)` for every run index, validates each solution and
/// aggregates min / max / avg of the objective.
pub fn run(inst: &Instance, runs: usize, mut method: impl FnMut(usize) -> Tour) -> Stats {
    let mut min = i32::MAX;
    let mut max = i32::MIN;
    let mut sum = 0i64;
    let mut best = Tour::new();

    for r in 0..runs {
        let tour = method(r);
        if let Err(e) = validate(inst, &tour) {
            panic!("run {r} on {} produced an infeasible solution: {e}", inst.name);
        }
        let value = objective(inst, &tour);
        sum += value as i64;
        max = max.max(value);
        if value < min {
            min = value;
            best = tour;
        }
    }

    Stats { min, max, avg: sum as f64 / runs as f64, best }
}
