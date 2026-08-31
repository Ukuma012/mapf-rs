# mapf-rs

Multi-agent path finding (MAPF) algorithms in Rust.

## Algorithms

- **CBS** — Guni Sharon, Roni Stern, Ariel Felner, Nathan R. Sturtevant,
  "Conflict-based search for optimal multi-agent pathfinding",
  _Artificial Intelligence_ 219 (2015)
- **PIBT** — Keisuke Okumura, Manao Machida, Xavier Défago, Yasumasa Tamura,
  "Priority Inheritance with Backtracking for Iterative Multi-agent Path
  Finding", _IJCAI_ 2019
- **LaCAM** — Keisuke Okumura, "LaCAM: Search-Based Algorithm for Quick
  Multi-Agent Pathfinding", _AAAI_ 2023, and "Improving LaCAM for Scalable
  Eventually Optimal Multi-Agent Pathfinding", _IJCAI_ 2023

## Usage

```rust
use std::sync::Arc;

use mapf_rs::algorithms::lacam;
use mapf_rs::base::{Deadline, Graph, Instance};

fn main() {
    let graph = Arc::new(Graph::from_grid(8, 8, &[true; 64]));
    let instance = Instance::random(graph, 20, 0).unwrap();

    let solution = lacam::solve(&instance, 0, &Deadline::from_secs_f32(1.0)).unwrap();

    assert!(solution.validate(&instance).is_ok());
    println!("makespan {} soc {}", solution.makespan(), solution.sum_of_costs());
}
```

Solutions can be written in the format read by
[mapf-visualizer](https://github.com/Kei18/mapf-visualizer):

```sh
cargo run --release --example visualize
mapf-visualizer assets/random-32-32-10.map results/solution.txt
```

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>
