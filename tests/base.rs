use std::sync::Arc;

use mapf_rs::base::{
    AgentId, Conflict, Error, Graph, Instance, Solution, ValidationError, VertexId,
};

fn v(i: u32) -> VertexId {
    VertexId(i)
}

// ---- Graph ----

#[test]
fn from_grid_is_row_major_and_true_is_passable() {
    // 3x2 grid with a single obstacle at the top right
    let g = Graph::from_grid(3, 2, &[true, true, false, true, true, true]);
    assert_eq!(g.to_string(), "..@\n...");
    assert_eq!(g.num_vertices(), 5);
}

fn sorted(vs: &[VertexId]) -> Vec<VertexId> {
    let mut v = vs.to_vec();
    v.sort();
    v
}

#[test]
fn neighbors_are_the_adjacent_passable_cells() {
    // 3x3 with the centre blocked
    #[rustfmt::skip]
    let g = Graph::from_grid(3, 3, &[
        true, true,  true,
        true, false, true,
        true, true,  true,
    ]);
    let at = |x, y| g.vertex_at(x, y).unwrap();

    assert_eq!(sorted(g.neighbors(at(0, 0))), sorted(&[at(1, 0), at(0, 1)]));
    // (1,1) is blocked, so the top edge cell has only two neighbors
    assert_eq!(sorted(g.neighbors(at(1, 0))), sorted(&[at(0, 0), at(2, 0)]));
}

#[test]
fn actions_are_the_neighbors_plus_the_vertex_itself() {
    let g = Graph::from_grid(3, 1, &[true; 3]);
    for x in 0..3 {
        let v = g.vertex_at(x, 0).unwrap();
        let mut expected = g.neighbors(v).to_vec();
        expected.push(v);
        assert_eq!(sorted(g.actions(v)), sorted(&expected));
    }
}

#[test]
fn vertex_at_and_coord_round_trip() {
    let g = Graph::from_movingai_file("assets/random-32-32-10.map").unwrap();
    let mut seen = 0;
    for y in 0..g.height() {
        for x in 0..g.width() {
            if let Some(v) = g.vertex_at(x, y) {
                assert_eq!(g.coord(v), (x as u16, y as u16));
                seen += 1;
            }
        }
    }
    assert_eq!(seen, g.num_vertices());
}

#[test]
fn vertex_at_rejects_out_of_range() {
    let g = Graph::from_grid(3, 2, &[true; 6]);
    assert!(g.vertex_at(2, 1).is_some());
    assert!(g.vertex_at(3, 0).is_none());
    assert!(g.vertex_at(0, 2).is_none());
}

#[test]
#[should_panic]
fn from_grid_rejects_mismatched_length() {
    Graph::from_grid(3, 2, &[true; 5]);
}

#[test]
fn movingai_file_matches_from_grid() {
    let g = Graph::from_movingai_file("assets/random-32-32-10.map").unwrap();
    assert_eq!((g.width(), g.height()), (32, 32));
    assert_eq!(g.num_vertices(), 922);
}

// ---- Instance ----

#[test]
fn instance_rejects_malformed_input() {
    let g = Arc::new(Graph::from_grid(3, 1, &[true; 3]));
    // length mismatch
    assert!(Instance::new(g.clone(), vec![v(0), v(1)], vec![v(2)]).is_err());
    // vertex out of range
    assert!(Instance::new(g.clone(), vec![v(99)], vec![v(0)]).is_err());
    // duplicate start
    assert!(Instance::new(g.clone(), vec![v(0), v(0)], vec![v(1), v(2)]).is_err());
    // duplicate goal
    assert!(Instance::new(g.clone(), vec![v(0), v(1)], vec![v(2), v(2)]).is_err());
    // valid
    assert!(Instance::new(g, vec![v(0), v(1)], vec![v(2), v(0)]).is_ok());
}

#[test]
fn scen_reports_when_too_few_agents() {
    let g = Arc::new(Graph::from_movingai_file("assets/random-32-32-10.map").unwrap());
    let scen = "assets/random-32-32-10-random-1.scen";
    assert_eq!(
        Instance::from_scen_file(g.clone(), scen, 10)
            .unwrap()
            .num_agents(),
        10
    );
    match Instance::from_scen_file(g, scen, 100_000) {
        Err(Error::NotEnoughAgents { requested, .. }) => assert_eq!(requested, 100_000),
        other => panic!("expected NotEnoughAgents, got {other:?}"),
    }
}

#[test]
fn random_instance_is_valid_and_reproducible() {
    let g = Arc::new(Graph::from_movingai_file("assets/random-32-32-10.map").unwrap());
    let a = Instance::random(g.clone(), 100, 7).unwrap();
    let b = Instance::random(g.clone(), 100, 7).unwrap();
    assert_eq!(a.starts(), b.starts());
    assert_eq!(a.goals(), b.goals());
    assert_ne!(
        a.starts(),
        Instance::random(g.clone(), 100, 8).unwrap().starts()
    );
    assert!(Instance::random(g, 10_000, 0).is_err());
}

// ---- Solution ----

#[test]
fn sum_of_costs_and_loss_differ_when_an_agent_leaves_its_goal() {
    // One agent on the line 0-1-2 with goal 2: it arrives, is pushed back
    // to 1, then returns.  Timesteps off the goal are 0 and 3.
    let solution = Solution::from_configs(vec![
        vec![v(1)],
        vec![v(2)],
        vec![v(2)],
        vec![v(1)],
        vec![v(2)],
    ]);
    assert_eq!(solution.makespan(), 4);

    // cost is the timestep it finally settles on the goal
    assert_eq!(solution.path_cost(AgentId(0)), 4);
    assert_eq!(solution.sum_of_costs(), 4);

    // loss counts moves with an endpoint off the goal, so the 1 -> 2
    // transition where it stays on the goal is not counted
    assert_eq!(solution.sum_of_loss(), 3);
}

#[test]
fn metrics_agree_when_an_agent_never_leaves_its_goal() {
    let solution = Solution::from_configs(vec![vec![v(0)], vec![v(1)], vec![v(2)], vec![v(2)]]);
    assert_eq!(solution.makespan(), 3);
    assert_eq!(solution.path_cost(AgentId(0)), 2);
    assert_eq!(solution.sum_of_costs(), 2);
    assert_eq!(solution.sum_of_loss(), 2);
}

#[test]
fn from_paths_pads_with_goal() {
    let solution = Solution::from_paths(vec![vec![v(0), v(1), v(2)], vec![v(5)]]);
    assert_eq!(solution.len(), 3);
    assert_eq!(solution.config_at(2), &[v(2), v(5)]);
}

// ---- validate ----

fn line3() -> Arc<Graph> {
    Arc::new(Graph::from_grid(3, 1, &[true; 3]))
}

#[test]
fn validate_accepts_a_correct_solution() {
    let g = line3();
    let ins = Instance::new(g, vec![v(0), v(2)], vec![v(1), v(2)]).unwrap();
    let solution = Solution::from_configs(vec![vec![v(0), v(2)], vec![v(1), v(2)]]);
    assert_eq!(solution.validate(&ins), Ok(()));
}

#[test]
fn validate_detects_vertex_conflict() {
    let g = line3();
    let ins = Instance::new(g, vec![v(0), v(2)], vec![v(2), v(0)]).unwrap();
    let solution =
        Solution::from_configs(vec![vec![v(0), v(2)], vec![v(1), v(1)], vec![v(2), v(0)]]);
    assert!(matches!(
        solution.validate(&ins),
        Err(ValidationError::Conflict(Conflict::Vertex { t: 1, v, .. })) if v == VertexId(1)
    ));
}

#[test]
fn validate_detects_swap_conflict() {
    let g = line3();
    let ins = Instance::new(g, vec![v(0), v(1)], vec![v(1), v(0)]).unwrap();
    let solution = Solution::from_configs(vec![vec![v(0), v(1)], vec![v(1), v(0)]]);
    assert!(matches!(
        solution.validate(&ins),
        Err(ValidationError::Conflict(Conflict::Swap { t: 0, .. }))
    ));
}

#[test]
fn validate_detects_illegal_move() {
    let g = line3();
    let ins = Instance::new(g, vec![v(0)], vec![v(2)]).unwrap();
    let solution = Solution::from_configs(vec![vec![v(0)], vec![v(2)]]);
    assert!(matches!(
        solution.validate(&ins),
        Err(ValidationError::IllegalMove { t: 1, .. })
    ));
}

#[test]
fn validate_detects_wrong_start_and_unreached_goal() {
    let g = line3();
    let ins = Instance::new(g, vec![v(0)], vec![v(2)]).unwrap();
    assert!(matches!(
        Solution::from_configs(vec![vec![v(1)], vec![v(2)]]).validate(&ins),
        Err(ValidationError::WrongStart { .. })
    ));
    assert!(matches!(
        Solution::from_configs(vec![vec![v(0)], vec![v(1)]]).validate(&ins),
        Err(ValidationError::UnreachedGoal { .. })
    ));
    assert_eq!(
        Solution::from_configs(vec![]).validate(&ins),
        Err(ValidationError::Empty)
    );
}

// ---- mapf-visualizer output ----

#[test]
fn visualizer_output_is_parsable_by_the_reference_regex() {
    let g = Graph::from_grid(3, 1, &[true; 3]);
    let instance = Instance::new(g.clone(), vec![v(0), v(2)], vec![v(1), v(2)]).unwrap();
    let solution = Solution::from_configs(vec![vec![v(0), v(2)], vec![v(1), v(2)]]);
    assert_eq!(solution.validate(&instance), Ok(()));

    let mut out = Vec::new();
    solution.write_visualizer(&mut out, &g).unwrap();
    let text = String::from_utf8(out).unwrap();

    // mapf-visualizer only looks at lines containing ":(" and its regex
    // requires a comma after every "(x,y)", including the last one.
    let lines: Vec<&str> = text.lines().filter(|l| l.contains(":(")).collect();
    assert_eq!(lines.len(), solution.len());
    for (t, line) in lines.iter().enumerate() {
        let (prefix, rest) = line.split_once(':').unwrap();
        assert_eq!(prefix, t.to_string());
        assert!(rest.ends_with(','), "missing trailing comma: {line:?}");
        assert_eq!(rest.matches("),").count(), solution.num_agents());
    }

    assert_eq!(lines[0], "0:(0,0),(2,0),");
    assert_eq!(lines[1], "1:(1,0),(2,0),");
}
