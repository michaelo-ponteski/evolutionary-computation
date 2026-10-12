use crate::instance::Instance;
use crate::solution::{Tour, objective, validate};

pub struct Stats {
    pub min: i32,
    pub max: i32,
    pub avg: f64,
    pub median: f64,
    /// Averages over all runs of the two parts of the objective, and of the
    /// edge from the last node of the list back to the first.
    pub avg_length: f64,
    pub avg_cost: f64,
    pub avg_closing: f64,
    pub best: Tour,
}

/// Runs `method(run_index)` for every run index, validates each solution and
/// aggregates statistics of the objective.
pub fn run(inst: &Instance, runs: usize, mut method: impl FnMut(usize) -> Tour) -> Stats {
    let mut values = Vec::with_capacity(runs);
    let (mut cost_sum, mut closing_sum) = (0i64, 0i64);
    let mut best = Tour::new();
    let mut best_value = i32::MAX;

    for r in 0..runs {
        let tour = method(r);
        if let Err(e) = validate(inst, &tour) {
            panic!(
                "run {r} on {} produced an infeasible solution: {e}",
                inst.name
            );
        }
        let value = objective(inst, &tour);
        cost_sum += tour.iter().map(|&v| inst.costs[v] as i64).sum::<i64>();
        closing_sum += inst.d(tour[tour.len() - 1], tour[0]) as i64;
        if value < best_value {
            best_value = value;
            best = tour;
        }
        values.push(value);
    }

    let n = runs as f64;
    let sum: i64 = values.iter().map(|&v| v as i64).sum();
    let avg = sum as f64 / n;
    let avg_cost = cost_sum as f64 / n;

    let mut sorted = values.clone();
    sorted.sort_unstable();
    let mid = runs / 2;
    let median = if runs.is_multiple_of(2) {
        (sorted[mid - 1] as f64 + sorted[mid] as f64) / 2.0
    } else {
        sorted[mid] as f64
    };

    Stats {
        min: sorted[0],
        max: sorted[runs - 1],
        avg,
        median,
        avg_length: avg - avg_cost,
        avg_cost,
        avg_closing: closing_sum as f64 / n,
        best,
    }
}
