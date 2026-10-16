use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::time::Instant;

use rand::SeedableRng;
use rand::rngs::StdRng;

use common::experiment::{self, Stats};
use common::heuristics::greedy_cycle::greedy_cycle;
use common::heuristics::nearest_neighbor::{nn_anywhere, nn_end};
use common::heuristics::random::random_solution;
use common::heuristics::regret::{REGRET, WEIGHTED, regret_cycle, regret_nn};
use common::instance::Instance;
use common::solution::Tour;

const RANDOM_RUNS: usize = 200;
const SEED: u64 = 42;
/// Methods introduced in this lab; only their best tours are saved (the Lab 1
/// methods appear only in the summary table).
const NEW_METHODS: [&str; 4] = ["nn_regret", "nn_weighted", "cycle_regret", "cycle_weighted"];
/// Weights of the regret tried in the extra experiment (0 = plain greedy,
/// 0.5 = equal weights, 1 = pure 2-regret).
const LAMBDAS: [f64; 11] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

/// Runs one method, prints its line of the results table and returns the stats.
fn measure(method: &str, run: impl FnOnce() -> Stats) -> Stats {
    let t = Instant::now();
    let s = run();
    println!(
        "  {method:<14} min {:>7}  max {:>7}  avg {:>10.1}  ({:.0?})",
        s.min,
        s.max,
        s.avg,
        t.elapsed()
    );
    s
}

/// One node index per line, so the file can be pasted straight into the
/// "List of nodes" column of the solution checker.
fn tour_to_lines(tour: &Tour) -> String {
    tour.iter().map(|v| format!("{v}\n")).collect()
}

fn main() {
    let results_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("results");
    fs::create_dir_all(results_dir.join("best")).expect("cannot create results dir");
    let mut summary =
        String::from("instance,method,min,max,avg,median,avg_length,avg_cost,avg_closing\n");
    let mut weights = String::from("instance,base,lambda,min,max,avg\n");

    for file in ["TSPA.csv", "TSPB.csv"] {
        let inst = Instance::load(file).expect("failed to load instance");
        println!(
            "{} (n = {}, select {})",
            inst.name,
            inst.n,
            inst.select_count()
        );

        // Greedy methods: one run per starting node (n = 200 -> 200 solutions).
        // The lab 1 methods are rerun so the summary compares all of them.
        let mut rng = StdRng::seed_from_u64(SEED);
        let n = inst.n;
        let methods = [
            (
                "random",
                measure("random", || {
                    experiment::run(&inst, RANDOM_RUNS, |_| random_solution(&inst, &mut rng))
                }),
            ),
            ("nn_end", measure("nn_end", || experiment::run(&inst, n, |s| nn_end(&inst, s)))),
            (
                "nn_anywhere",
                measure("nn_anywhere", || experiment::run(&inst, n, |s| nn_anywhere(&inst, s))),
            ),
            (
                "greedy_cycle",
                measure("greedy_cycle", || experiment::run(&inst, n, |s| greedy_cycle(&inst, s))),
            ),
            (
                "nn_regret",
                measure("nn_regret", || {
                    experiment::run(&inst, n, |s| regret_nn(&inst, s, REGRET))
                }),
            ),
            (
                "nn_weighted",
                measure("nn_weighted", || {
                    experiment::run(&inst, n, |s| regret_nn(&inst, s, WEIGHTED))
                }),
            ),
            (
                "cycle_regret",
                measure("cycle_regret", || {
                    experiment::run(&inst, n, |s| regret_cycle(&inst, s, REGRET))
                }),
            ),
            (
                "cycle_weighted",
                measure("cycle_weighted", || {
                    experiment::run(&inst, n, |s| regret_cycle(&inst, s, WEIGHTED))
                }),
            ),
        ];

        for (method, s) in &methods {
            writeln!(
                summary,
                "{},{method},{},{},{:.2},{:.1},{:.2},{:.2},{:.2}",
                inst.name, s.min, s.max, s.avg, s.median, s.avg_length, s.avg_cost, s.avg_closing
            )
            .unwrap();
            if NEW_METHODS.contains(method) {
                let path = results_dir
                    .join("best")
                    .join(format!("{}_{method}.txt", inst.name));
                fs::write(path, tour_to_lines(&s.best)).expect("cannot write best tour");
            }
        }

        // Extra experiment: other weights of the regret in the weighted criterion.
        for lambda in LAMBDAS {
            for (base, s) in [
                ("nn", experiment::run(&inst, n, |s| regret_nn(&inst, s, lambda))),
                ("cycle", experiment::run(&inst, n, |s| regret_cycle(&inst, s, lambda))),
            ] {
                writeln!(
                    weights,
                    "{},{base},{lambda:.1},{},{},{:.2}",
                    inst.name, s.min, s.max, s.avg
                )
                .unwrap();
            }
        }
    }

    fs::write(results_dir.join("summary.csv"), summary).expect("cannot write summary");
    fs::write(results_dir.join("weights.csv"), weights).expect("cannot write weights");
    println!("results written to {}", results_dir.display());
}
