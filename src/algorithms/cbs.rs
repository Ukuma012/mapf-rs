//! CBS: Conflict-Based Search.
//!
//! Sharon et al., "Conflict-based search for optimal multi-agent pathfinding",
//! Artificial Intelligence 219 (2015). 

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::base::{
    AgentId, Conflict, Deadline, DistTable, Graph, Instance, Solution, VertexId,
    find_conflict_in_paths,
};

type Path = Vec<VertexId>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Constraint {
    Vertex {
        agent: AgentId,
        v: VertexId,
        t: usize,
    },
    Edge {
        agent: AgentId,
        from: VertexId,
        to: VertexId,
        t: usize,
    },
}

impl Constraint {
    fn agent(&self) -> AgentId {
        match self {
            Constraint::Vertex { agent, .. } | Constraint::Edge { agent, .. } => *agent,
        }
    }
}

fn branch(c: &Conflict) -> [Constraint; 2] {
    match *c {
        Conflict::Vertex { t, a, b, v } => [
            Constraint::Vertex { agent: a, v, t },
            Constraint::Vertex { agent: b, v, t },
        ],
        Conflict::Swap { t, a, b, from, to } => [
            Constraint::Edge {
                agent: a,
                from,
                to,
                t,
            },
            Constraint::Edge {
                agent: b,
                from: to,
                to: from,
                t,
            },
        ],
    }
}

struct CtNode {
    constraints: Vec<Constraint>,
    paths: Vec<Path>,
    soc: usize,
}

impl PartialEq for CtNode {
    fn eq(&self, other: &Self) -> bool {
        self.soc == other.soc && self.constraints.len() == other.constraints.len()
    }
}
impl Eq for CtNode {}
impl Ord for CtNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .soc
            .cmp(&self.soc)
            .then(self.constraints.len().cmp(&other.constraints.len()))
    }
}
impl PartialOrd for CtNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct AstarNode {
    f: usize,
    t: usize,
    v: VertexId,
}

impl PartialEq for AstarNode {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f && self.t == other.t
    }
}
impl Eq for AstarNode {}
impl Ord for AstarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f).then(self.t.cmp(&other.t))
    }
}
impl PartialOrd for AstarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn low_level(
    graph: &Graph,
    dist: &mut DistTable,
    agent: AgentId,
    start: VertexId,
    goal: VertexId,
    constraints: &[Constraint],
) -> Option<Path> {
    let mut blocked_vertex: HashSet<(VertexId, usize)> = HashSet::new();
    let mut blocked_edge: HashSet<(VertexId, VertexId, usize)> = HashSet::new();
    let mut latest_goal: Option<usize> = None;
    let mut latest_t = 0;

    for c in constraints.iter().filter(|c| c.agent() == agent) {
        match *c {
            Constraint::Vertex { v, t, .. } => {
                blocked_vertex.insert((v, t));
                if v == goal {
                    latest_goal = Some(latest_goal.map_or(t, |lt: usize| lt.max(t)));
                }
                latest_t = latest_t.max(t);
            }
            Constraint::Edge { from, to, t, .. } => {
                blocked_edge.insert((from, to, t));
                latest_t = latest_t.max(t + 1);
            }
        }
    }

    let horizon = latest_t + graph.num_vertices();
    let h0 = dist.get(goal, start).value()? as usize;

    let mut open = BinaryHeap::new();
    let mut parents: HashMap<(VertexId, usize), (VertexId, usize)> = HashMap::new();
    let mut closed: HashSet<(VertexId, usize)> = HashSet::new();

    open.push(AstarNode {
        f: h0,
        t: 0,
        v: start,
    });
    closed.insert((start, 0));

    while let Some(AstarNode { t, v, .. }) = open.pop() {
        if v == goal && latest_goal.is_none_or(|lt| t > lt) {
            let mut path = vec![v];
            let mut cur = (v, t);
            while cur != (start, 0) {
                cur = parents[&cur];
                path.push(cur.0);
            }
            path.reverse();
            return Some(path);
        }
        if t >= horizon {
            continue;
        }

        for &nv in graph.actions(v) {
            let nt = t + 1;
            if blocked_vertex.contains(&(nv, nt)) || blocked_edge.contains(&(v, nv, t)) {
                continue;
            }
            if !closed.insert((nv, nt)) {
                continue;
            }
            let Some(h) = dist.get(goal, nv).value() else {
                continue;
            };
            parents.insert((nv, nt), (v, t));
            open.push(AstarNode {
                f: nt + h as usize,
                t: nt,
                v: nv,
            });
        }
    }
    None
}

fn soc_of(paths: &[Path]) -> usize {
    paths.iter().map(|p| p.len() - 1).sum()
}

pub fn solve(ins: &Instance, deadline: &Deadline) -> Option<Solution> {
    let graph = ins.graph();
    let mut dist = DistTable::new_parallel(graph, ins.goals());

    let mut paths = Vec::with_capacity(ins.num_agents());
    for a in 0..ins.num_agents() {
        let agent = AgentId(a as u32);
        paths.push(low_level(
            graph,
            &mut dist,
            agent,
            ins.starts()[a],
            ins.goals()[a],
            &[],
        )?);
    }

    let mut open = BinaryHeap::new();
    open.push(CtNode {
        soc: soc_of(&paths),
        constraints: Vec::new(),
        paths,
    });

    while let Some(node) = open.pop() {
        if deadline.expired() {
            return None;
        }

        let Some(conflict) = find_conflict_in_paths(&node.paths) else {
            return Some(Solution::from_paths(node.paths));
        };

        for constraint in branch(&conflict) {
            let a = constraint.agent().index();
            let mut constraints = node.constraints.clone();
            constraints.push(constraint);

            let Some(path) = low_level(
                graph,
                &mut dist,
                constraint.agent(),
                ins.starts()[a],
                ins.goals()[a],
                &constraints,
            ) else {
                continue;
            };

            let mut paths = node.paths.clone();
            paths[a] = path;
            open.push(CtNode {
                soc: soc_of(&paths),
                constraints,
                paths,
            });
        }
    }
    None
}
