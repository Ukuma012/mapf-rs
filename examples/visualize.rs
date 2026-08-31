//! Writes a solution in mapf-visualizer format.
//!
//! ```sh
//! cargo run --release --example visualize
//! mapf-visualizer assets/random-32-32-10.map results/solution.txt
//! ```

use std::sync::Arc;

use mapf_rs::algorithms::lacam;
use mapf_rs::base::{Deadline, Graph, Instance};

fn main() -> mapf_rs::base::Result<()> {
    let map = "assets/random-32-32-10.map";
    let graph = Arc::new(Graph::from_movingai_file(map)?);
    let instance =
        Instance::from_scen_file(graph.clone(), "assets/random-32-32-10-random-1.scen", 100)?;

    let solution = lacam::solve(&instance, 0, &Deadline::from_secs_f32(10.0)).unwrap();
    assert!(solution.validate(&instance).is_ok());

    std::fs::create_dir_all("results")?;
    solution.write_visualizer_file("results/solution.txt", &graph)?;

    println!("wrote results/solution.txt");
    println!("run: mapf-visualizer {map} results/solution.txt");
    Ok(())
}
