# mapf-rs

Multi-agent path finding (MAPF) algorithms in Rust.

## Algorithms

- **CBS** — Sharon, Stern, Felner, Sturtevant, "Conflict-based search for
  optimal multi-agent pathfinding", _Artificial Intelligence_ 219 (2015)
- **PIBT** — Okumura, Machida, Défago, Tamura, "Priority Inheritance with
  Backtracking for Iterative Multi-agent Path Finding", IJCAI 2019

## Usage

```rust
use std::sync::Arc;

use mapf_rs::algorithms::pibt;
use mapf_rs::base::{Deadline, Graph, Instance};

fn main() {
    let graph = Arc::new(Graph::from_grid(8, 8, &[true; 64]));
    let instance = Instance::random(graph, 20, 0).unwrap();

    let solution = pibt::solve(&instance, 0, &Deadline::from_secs_f32(1.0)).unwrap();

    assert!(solution.validate(&instance).is_ok());
    println!("makespan {} soc {}", solution.makespan(), solution.sum_of_costs());
}
```

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>
