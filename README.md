# Evolutionary Computation — labs

TSP variant with node costs: select exactly 50% of the nodes (rounded up) and
build a Hamiltonian cycle over them, minimising cycle length + sum of node costs.
Distances are Euclidean, rounded to the nearest integer.

Each lab's state is frozen under a git tag (`lab1`, `lab2`, …).

## Layout

```
problem/   instances TSPA.csv, TSPB.csv (x;y;cost per node)
common/    shared library: instance + distance matrix, objective, validation,
           experiment runner, heuristics
lab1/      greedy heuristics (random, nearest neighbour ×2, greedy cycle)
```

## Running

Requires Rust (stable, edition 2024).

```sh
cargo test                     # objective checked against the course solution checker
cargo run --release -p lab1    # run the lab 1 experiment
```
