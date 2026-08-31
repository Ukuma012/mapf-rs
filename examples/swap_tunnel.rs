use std::sync::Arc;

use mapf_rs::algorithms::pibt;
use mapf_rs::base::{Deadline, Graph, Instance};

fn main() -> mapf_rs::base::Result<()> {
    let graph = Arc::new(Graph::from_movingai_file("assets/tunnel.map")?);
    println!("{graph}\n");

    let a = graph.vertex_at(0, 0).unwrap();
    let b = graph.vertex_at(0, 5).unwrap();
    let ins = Instance::new(graph.clone(), vec![a, b], vec![b, a])?;

    let deadline = Deadline::from_secs_f32(3.0);
    match pibt::solve(&ins, 0, &deadline) {
        Some(solution) => {
            println!("solved: makespan={} soc={}", solution.makespan(), solution.sum_of_costs());
            for t in 0..solution.len() {
                let c = solution.config_at(t);
                let p: Vec<_> = c.iter().map(|&v| graph.coord(v)).collect();
                println!("  t={t}: {p:?}");
            }
        }
        None => println!("FAILED (deadlock) after {:.0}ms", deadline.elapsed_ms()),
    }
    Ok(())
}
