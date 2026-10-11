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
use common::instance::Instance;
use common::solution::Tour;

const RANDOM_RUNS: usize = 200;
const SEED: u64 = 42;

/// Runs one method, prints its line of the results table and returns the stats.
fn measure(method: &str, run: impl FnOnce() -> Stats) -> Stats {
    let t = Instant::now();
    let s = run();
    println!(
        "  {method:<12} min {:>7}  max {:>7}  avg {:>10.1}  ({:.0?})",
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
    let mut summary = String::from("instance,method,min,max,avg\n");

    for file in ["TSPA.csv", "TSPB.csv"] {
        let inst = Instance::load(file).expect("failed to load instance");
        println!(
            "{} (n = {}, select {})",
            inst.name,
            inst.n,
            inst.select_count()
        );

        // Greedy methods: one run per starting node (n = 200 -> 200 solutions).
        let mut rng = StdRng::seed_from_u64(SEED);
        let methods = [
            (
                "random",
                measure("random", || {
                    experiment::run(&inst, RANDOM_RUNS, |_| random_solution(&inst, &mut rng))
                }),
            ),
            (
                "nn_end",
                measure("nn_end", || {
                    experiment::run(&inst, inst.n, |s| nn_end(&inst, s))
                }),
            ),
            (
                "nn_anywhere",
                measure("nn_anywhere", || {
                    experiment::run(&inst, inst.n, |s| nn_anywhere(&inst, s))
                }),
            ),
            (
                "greedy_cycle",
                measure("greedy_cycle", || {
                    experiment::run(&inst, inst.n, |s| greedy_cycle(&inst, s))
                }),
            ),
        ];

        for (method, s) in &methods {
            writeln!(
                summary,
                "{},{method},{},{},{:.2}",
                inst.name, s.min, s.max, s.avg
            )
            .unwrap();
            let path = results_dir
                .join("best")
                .join(format!("{}_{method}.txt", inst.name));
            fs::write(path, tour_to_lines(&s.best)).expect("cannot write best tour");
        }
    }

    fs::write(results_dir.join("summary.csv"), summary).expect("cannot write summary");
    println!("results written to {}", results_dir.display());
}
