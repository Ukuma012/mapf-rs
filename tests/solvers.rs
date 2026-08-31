use std::sync::Arc;

use mapf_rs::algorithms::{cbs, pibt};
use mapf_rs::base::{Deadline, Graph, Instance};

fn map() -> Arc<Graph> {
    Arc::new(Graph::from_movingai_file("assets/random-32-32-10.map").unwrap())
}

const SCEN: &str = "assets/random-32-32-10-random-1.scen";

fn dl() -> Deadline {
    Deadline::from_secs_f32(30.0)
}

#[test]
fn pibt_solutions_are_valid() {
    let g = map();
    for k in [1, 10, 50, 200, 400] {
        let ins = Instance::from_scen_file(g.clone(), SCEN, k).unwrap();
        for seed in 0..3 {
            let solution = pibt::solve(&ins, seed, &dl()).expect("pibt should solve");
            assert_eq!(solution.validate(&ins), Ok(()), "k={k} seed={seed}");
        }
    }
}

#[test]
fn pibt_is_deterministic_for_a_seed() {
    let g = map();
    let ins = Instance::from_scen_file(g, SCEN, 100).unwrap();
    let a = pibt::solve(&ins, 3, &dl()).unwrap();
    let b = pibt::solve(&ins, 3, &dl()).unwrap();
    assert_eq!(a.configs(), b.configs());
}

#[test]
fn pibt_solves_random_instances() {
    let g = map();
    for seed in 0..5 {
        let ins = Instance::random(g.clone(), 300, seed).unwrap();
        let solution = pibt::solve(&ins, seed, &dl()).expect("pibt should solve");
        assert_eq!(solution.validate(&ins), Ok(()), "seed={seed}");
    }
}

#[test]
fn cbs_solutions_are_valid() {
    let g = map();
    for k in [1, 5, 10, 20] {
        let ins = Instance::from_scen_file(g.clone(), SCEN, k).unwrap();
        let solution = cbs::solve(&ins, &dl()).expect("cbs should solve");
        assert_eq!(solution.validate(&ins), Ok(()), "k={k}");
    }
}

/// Optimal sum-of-costs for 3 agents on a small grid, computed once by an
/// exhaustive Dijkstra over the joint configuration space.  The numbers do
/// not come from CBS, so this checks optimality and not just regression.
#[test]
fn cbs_returns_the_true_optimum() {
    #[rustfmt::skip]
    let g = Arc::new(Graph::from_grid(4, 4, &[
        true,  true,  true,  true,
        true,  false, true,  false,
        true,  true,  true,  true,
        false, true,  false, true,
    ]));

    let expected = [6, 13, 11, 9, 10, 13, 12, 13, 2, 11, 16, 12];
    for (seed, expected) in expected.into_iter().enumerate() {
        let instance = Instance::random(g.clone(), 3, seed as u64).unwrap();
        let solution = cbs::solve(&instance, &dl()).expect("cbs should solve");
        assert_eq!(solution.validate(&instance), Ok(()), "seed={seed}");
        assert_eq!(solution.sum_of_costs(), expected, "seed={seed}");
    }
}

#[test]
fn pibt_swap_resolves_tunnel_deadlock() {
    let g = Arc::new(Graph::from_movingai_file("assets/tunnel.map").unwrap());
    let a = g.vertex_at(0, 0).unwrap();
    let b = g.vertex_at(0, 5).unwrap();
    let ins = Instance::new(g, vec![a, b], vec![b, a]).unwrap();

    for seed in 0..10 {
        let solution = pibt::solve(&ins, seed, &Deadline::from_secs_f32(3.0))
            .unwrap_or_else(|| panic!("deadlock at seed={seed}"));
        assert_eq!(solution.validate(&ins), Ok(()));
    }
}
