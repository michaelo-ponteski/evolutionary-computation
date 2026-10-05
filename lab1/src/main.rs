use rand::SeedableRng;
use rand::rngs::StdRng;

use common::experiment::{self, Stats};
use common::heuristics::random::random_solution;
use common::instance::Instance;

const RUNS: usize = 200;
const SEED: u64 = 42;

fn report(method: &str, s: &Stats) {
    println!("  {method:<10} min {:>7}  max {:>7}  avg {:>10.1}", s.min, s.max, s.avg);
}

fn main() {
    for file in ["TSPA.csv", "TSPB.csv"] {
        let inst = Instance::load(file).expect("failed to load instance");
        println!("{} (n = {}, select {})", inst.name, inst.n, inst.select_count());

        let mut rng = StdRng::seed_from_u64(SEED);
        let random = experiment::run(&inst, RUNS, |_| random_solution(&inst, &mut rng));
        report("random", &random);
    }
}
