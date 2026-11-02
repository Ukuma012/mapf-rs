//! PP: Prioritized Planning
//!
//! Erdmann and Lozano-Perez, "On multiple moving objects", https://people.csail.mit.edu/tlp/publications/01087401.pdf
//! Silver, D. (2005). Cooperative Pathfinding. Proceedings of the AAAI Conference on Artificial Intelligence and Interactive Digital Entertainment, 1(1), 117–122. https://doi.org/10.1609/aiide.v1i1.18726

use crate::base::{AgentId, Deadline, DistTable, Graph, Instance, Rng, Solution, VertexId};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

type Path = Vec<VertexId>;

/// The space-time cells claimed by the agents planned so far.
#[derive(Default)]
struct Reservation {
    /// `(v, t)`: vertex `v` is taken at time `t`.
    occupied: HashSet<(VertexId, usize)>,
    /// `(from, to, t)`: moving `from` -> `to` over `t..t+1` would swap with
    /// an agent traveling the other way.
    swaps: HashSet<(VertexId, VertexId, usize)>,
    /// `v -> t`: an agent reached its goal `v` at time `t` and never moves again.
    parked: HashMap<VertexId, usize>,
    /// `v -> t`: the last time any planned path touches `v`.
    last_use: HashMap<VertexId, usize>,
}

impl Reservation {
    fn add(&mut self, path: &Path) {
        for (t, &v) in path.iter().enumerate() {
            self.occupied.insert((v, t));
            self.last_use
                .entry(v)
                .and_modify(|last| *last = (*last).max(t))
                .or_insert(t);
        }
        for t in 0..path.len() - 1 {
            self.swaps.insert((path[t + 1], path[t], t));
        }
        let arrival = path.len() - 1;
        self.parked.insert(path[arrival], arrival);
    }

    fn blocks(&self, v: VertexId, t: usize) -> bool {
        self.occupied.contains(&(v, t)) || self.parked.get(&v).is_some_and(|&since| t >= since)
    }

    fn blocks_move(&self, from: VertexId, to: VertexId, t: usize) -> bool {
        self.swaps.contains(&(from, to, t))
    }
}

struct Node {
    f: usize,
    t: usize,
    v: VertexId,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f && self.t == other.t
    }
}
impl Eq for Node {}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f).then(self.t.cmp(&other.t))
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Space-time A* for one agent, avoiding everything in `res`
fn plan(
    graph: &Graph,
    dist: &mut DistTable,
    start: VertexId,
    goal: VertexId,
    res: &Reservation,
    deadline: &Deadline,
) -> Option<Path> {
    let h0 = dist.get(goal, start).value()? as usize;
    // The earliest time this agent may come to rest on its goal: by then every
    // planned path has left it.
    let settle = res.last_use.get(&goal).map_or(0, |&last| last + 1);

    let upper_t = (h0 + graph.num_vertices()).max(settle);

    let mut open = BinaryHeap::new();
    let mut parents: HashMap<(VertexId, usize), (VertexId, usize)> = HashMap::new();
    let mut closed: HashSet<(VertexId, usize)> = HashSet::new();

    open.push(Node {
        f: h0.max(settle),
        t: 0,
        v: start,
    });
    closed.insert((start, 0));

    while let Some(Node { t, v, .. }) = open.pop() {
        if deadline.expired() {
            return None;
        }

        if v == goal && t >= settle {
            let mut path = vec![v];
            let mut cur = (v, t);
            while cur != (start, 0) {
                cur = parents[&cur];
                path.push(cur.0);
            }
            path.reverse();
            return Some(path);
        }
        if t >= upper_t {
            continue;
        }

        for &nv in graph.actions(v) {
            let nt = t + 1;
            if res.blocks(nv, nt) || res.blocks_move(v, nv, t) {
                continue;
            }
            if !closed.insert((nv, nt)) {
                continue;
            }
            let Some(h) = dist.get(goal, nv).value() else {
                continue;
            };
            parents.insert((nv, nt), (v, t));
            open.push(Node {
                f: (nt + h as usize).max(settle),
                t: nt,
                v: nv,
            });
        }
    }
    None
}

fn plan_all(
    ins: &Instance,
    order: &[AgentId],
    dist: &mut DistTable,
    deadline: &Deadline,
) -> Option<Vec<Path>> {
    let mut res = Reservation::default();
    let mut paths = vec![Path::new(); ins.num_agents()];

    for &a in order {
        let i = a.index();
        let path = plan(
            ins.graph(),
            dist,
            ins.starts()[i],
            ins.goals()[i],
            &res,
            deadline,
        )?;
        res.add(&path);
        paths[i] = path;
    }
    Some(paths)
}

/// Plans the agents one at a time in `order`.
pub fn solve_with_order(
    ins: &Instance,
    order: &[AgentId],
    deadline: &Deadline,
) -> Option<Solution> {
    debug_assert_eq!(order.len(), ins.num_agents());
    let mut dist = DistTable::new_parallel(ins.graph(), ins.goals());
    plan_all(ins, order, &mut dist, deadline).map(Solution::from_paths)
}

/// Retries [`solve_with_order`] with reshuffled priorities until `deadline`.
pub fn solve(ins: &Instance, seed: u64, deadline: &Deadline) -> Option<Solution> {
    let mut dist = DistTable::new_parallel(ins.graph(), ins.goals());
    let mut rng = Rng::new(seed);
    let mut order: Vec<AgentId> = (0..ins.num_agents() as u32).map(AgentId).collect();

    loop {
        if deadline.expired() {
            return None;
        }
        if let Some(paths) = plan_all(ins, &order, &mut dist, deadline) {
            return Some(Solution::from_paths(paths));
        }
        rng.shuffle(&mut order);
    }
}
