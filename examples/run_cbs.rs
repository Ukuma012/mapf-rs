use std::sync::Arc;

use mapf_rs::algorithms::cbs;
use mapf_rs::base::{Deadline, Graph, Instance};

fn main() -> mapf_rs::base::Result<()> {
    let graph = Arc::new(Graph::from_movingai_file("assets/random-32-32-10.map")?);
    println!("{graph:?}");

    for k in [2, 5, 10, 15, 20, 25] {
        let ins =
            Instance::from_scen_file(graph.clone(), "assets/random-32-32-10-random-1.scen", k)?;
        let deadline = Deadline::from_secs_f32(10.0);
        match cbs::solve(&ins, &deadline) {
            Some(sol) => println!(
                "k={k:<4} solved  makespan={:<5} soc={:<7} loss={:<7} {:.1}ms",
                sol.makespan(),
                sol.sum_of_costs(),
                sol.sum_of_loss(),
                deadline.elapsed_ms()
            ),
            None => println!("k={k:<4} failed  {:.1}ms", deadline.elapsed_ms()),
        }
    }
    Ok(())
}
