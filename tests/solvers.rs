use std::sync::Arc;

use mapf_rs::algorithms::{cbs, lacam, pibt, pp};
use mapf_rs::base::{AgentId, Deadline, Graph, Instance};

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

#[test]
fn lacam_solutions_are_valid() {
    let g = map();
    for k in [1, 10, 50, 200, 400] {
        let instance = Instance::from_scen_file(g.clone(), SCEN, k).unwrap();
        for seed in 0..3 {
            let solution = lacam::solve(&instance, seed, &dl()).expect("lacam should solve");
            assert_eq!(solution.validate(&instance), Ok(()), "k={k} seed={seed}");

            let solution = lacam::solve_star(&instance, seed, &Deadline::from_secs_f32(0.2))
                .expect("lacam* should solve");
            assert_eq!(
                solution.validate(&instance),
                Ok(()),
                "star k={k} seed={seed}"
            );
        }
    }
}

#[test]
fn lacam_star_is_never_worse_than_lacam() {
    let g = map();
    for k in [10, 20, 50] {
        let instance = Instance::from_scen_file(g.clone(), SCEN, k).unwrap();
        let plain = lacam::solve(&instance, 0, &dl()).unwrap().sum_of_costs();
        let star = lacam::solve_star(&instance, 0, &Deadline::from_secs_f32(1.0))
            .unwrap()
            .sum_of_costs();
        assert!(star <= plain, "k={k}: lacam* {star} > lacam {plain}");
    }
}

#[test]
fn lacam_is_deterministic_for_a_seed() {
    let g = map();
    let instance = Instance::from_scen_file(g, SCEN, 100).unwrap();
    let a = lacam::solve(&instance, 3, &dl()).unwrap();
    let b = lacam::solve(&instance, 3, &dl()).unwrap();
    assert_eq!(a.configs(), b.configs());
}

#[test]
fn pp_solutions_are_valid() {
    let g = map();
    for k in [1, 10, 50, 100] {
        let ins = Instance::from_scen_file(g.clone(), SCEN, k).unwrap();
        for seed in 0..3 {
            let solution = pp::solve(&ins, seed, &dl()).expect("pp should solve");
            assert_eq!(solution.validate(&ins), Ok(()), "k={k} seed={seed}");
        }
    }
}

#[test]
fn pp_is_deterministic_for_a_seed() {
    let g = map();
    let ins = Instance::from_scen_file(g, SCEN, 100).unwrap();
    let a = pp::solve(&ins, 3, &dl()).unwrap();
    let b = pp::solve(&ins, 3, &dl()).unwrap();
    assert_eq!(a.configs(), b.configs());
}

#[test]
fn pp_is_never_better_than_cbs() {
    let g = map();
    for k in [1, 5, 10, 20] {
        let ins = Instance::from_scen_file(g.clone(), SCEN, k).unwrap();
        let opt = cbs::solve(&ins, &dl()).unwrap().sum_of_costs();
        let soc = pp::solve(&ins, 0, &dl()).unwrap().sum_of_costs();
        assert!(soc >= opt, "k={k}: pp {soc} < cbs {opt}");
    }
}

/// On `tunnel.map` the only passing bay sits beside one end of the corridor,
/// so the priority order decides whether the instance is solvable at all.
#[test]
fn pp_depends_on_the_priority_order() {
    let g = Arc::new(Graph::from_movingai_file("assets/tunnel.map").unwrap());
    let a = g.vertex_at(0, 0).unwrap(); // beside the bay at y=1
    let b = g.vertex_at(0, 5).unwrap(); // far end of the corridor
    let ins = Instance::new(g, vec![a, b], vec![b, a]).unwrap();

    // Agent 0 first: it sweeps the corridor, and agent 1 is left at the far
    // end with no bay behind it to step aside into.
    assert!(pp::solve_with_order(&ins, &[AgentId(0), AgentId(1)], &dl()).is_none());

    // Agent 1 first: agent 0 ducks into the bay, waits for it to pass, and
    // then walks the corridor it just vacated.
    let solution = pp::solve_with_order(&ins, &[AgentId(1), AgentId(0)], &dl()).unwrap();
    assert_eq!(solution.validate(&ins), Ok(()));
    assert_eq!(solution.path_cost(AgentId(1)), 5);
    assert!(solution.path_cost(AgentId(0)) > 5);

    // Restarting recovers the order that works.
    assert!(pp::solve(&ins, 0, &Deadline::from_secs_f32(1.0)).is_some());
}

#[test]
fn pp_cannot_swap_in_a_one_wide_corridor() {
    let g = Arc::new(Graph::from_grid(3, 1, &[true, true, true]));
    let a = g.vertex_at(0, 0).unwrap();
    let b = g.vertex_at(2, 0).unwrap();
    let ins = Instance::new(g, vec![a, b], vec![b, a]).unwrap();

    assert!(pp::solve(&ins, 0, &Deadline::from_secs_f32(0.2)).is_none());
}
