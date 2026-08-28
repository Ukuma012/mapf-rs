use mapf_rs::base::{Graph, Instance};

fn main() -> mapf_rs::base::Result<()> {
    let g = Graph::from_movingai_file("assets/random-32-32-10.map")?;
    println!("{g}");

    let ins = Instance::from_scen_file(g, "assets/random-32-32-10-random-1.scen", 400)?;

    Ok(())
}
