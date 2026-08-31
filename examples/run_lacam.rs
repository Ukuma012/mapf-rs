use std::sync::Arc;

use mapf_rs::algorithms::lacam;
use mapf_rs::base::{Deadline, Graph, Instance};

fn main() -> mapf_rs::base::Result<()> {
    let graph = Arc::new(Graph::from_movingai_file("assets/random-32-32-10.map")?);
    println!("{graph:?}");

    for k in [50, 100, 200, 300, 400] {
        let instance =
            Instance::from_scen_file(graph.clone(), "assets/random-32-32-10-random-1.scen", k)?;
        let deadline = Deadline::from_secs_f32(10.0);
        match lacam::solve(&instance, 0, &deadline) {
            Some(solution) => println!(
                "k={k:<4} solved  makespan={:<5} soc={:<7} loss={:<7} {:.1}ms  valid={}",
                solution.makespan(),
                solution.sum_of_costs(),
                solution.sum_of_loss(),
                deadline.elapsed_ms(),
                solution.validate(&instance).is_ok()
            ),
            None => println!("k={k:<4} failed  {:.1}ms", deadline.elapsed_ms()),
        }
    }
    Ok(())
}
