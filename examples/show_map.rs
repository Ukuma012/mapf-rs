use mapf_rs::base::Graph;

fn main() -> mapf_rs::base::Result<()> {
    let g = Graph::from_movingai_file("assets/random-32-32-10.map")?;
    println!("{g:?}");
    println!("{g}");

    Ok(())
}
