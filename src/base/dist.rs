use std::{collections::VecDeque, fmt, ops::Add};

use crate::base::{Graph, VertexId};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dist(u32);

impl Dist {
    pub const UNREACHABLE: Dist = Dist(u32::MAX);

    pub fn is_reachable(self) -> bool {
        self != Self::UNREACHABLE
    }

    pub fn value(self) -> Option<u32> {
        self.is_reachable().then_some(self.0)
    }
}

impl Add<u32> for Dist {
    type Output = Dist;
    fn add(self, rhs: u32) -> Dist {
        Dist(self.0.saturating_add(rhs))
    }
}

impl fmt::Debug for Dist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value() {
            Some(d) => write!(f, "{d}"),
            None => f.write_str("x"),
        }
    }
}

pub struct DistTable<'a> {
    graph: &'a Graph,
    tables: Vec<Option<GoalDist>>,
}

struct GoalDist {
    dist: Vec<Dist>,
    queue: VecDeque<VertexId>,
}

impl<'a> DistTable<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self {
            graph,
            tables: (0..graph.num_vertices()).map(|_| None).collect(),
        }
    }

    pub fn new_parallel(graph: &'a Graph, goals: &[VertexId]) -> Self {
        let mut table = Self::new(graph);
        let n = graph.num_vertices();

        let mut unique = goals.to_vec();
        unique.sort_unstable();
        unique.dedup();
        if unique.is_empty() {
            return table;
        }

        let workers = std::thread::available_parallelism()
            .map_or(1, |p| p.get())
            .min(unique.len());
        let chunk = unique.len().div_ceil(workers);

        let computed: Vec<(VertexId, Vec<Dist>)> = std::thread::scope(|s| {
            let handles: Vec<_> = unique
                .chunks(chunk)
                .map(|part| {
                    s.spawn(move || {
                        part.iter()
                            .map(|&g| (g, bfs_from(graph, g, n)))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            handles
                .into_iter()
                .flat_map(|h| h.join().unwrap())
                .collect()
        });

        for (g, dist) in computed {
            table.tables[g.index()] = Some(GoalDist {
                dist,
                queue: VecDeque::new(),
            });
        }
        table
    }

    pub fn get(&mut self, goal: VertexId, v: VertexId) -> Dist {
        let graph = self.graph;
        let n = graph.num_vertices();

        let gd = self.tables[goal.index()].get_or_insert_with(|| {
            let mut dist = vec![Dist::UNREACHABLE; n];
            dist[goal.index()] = Dist(0);
            let mut queue = VecDeque::new();
            queue.push_back(goal);
            GoalDist { dist, queue }
        });

        if gd.dist[v.index()] != Dist::UNREACHABLE {
            return gd.dist[v.index()];
        }

        while let Some(u) = gd.queue.pop_front() {
            let du = gd.dist[u.index()];
            for &w in graph.neighbors(u) {
                if gd.dist[w.index()] == Dist::UNREACHABLE {
                    gd.dist[w.index()] = du + 1;
                    gd.queue.push_back(w);
                }
            }
            if u == v {
                return du;
            }
        }

        Dist::UNREACHABLE
    }
}

fn bfs_from(graph: &Graph, goal: VertexId, n: usize) -> Vec<Dist> {
    let mut dist = vec![Dist::UNREACHABLE; n];
    dist[goal.index()] = Dist(0);
    let mut queue = VecDeque::new();
    queue.push_back(goal);
    while let Some(u) = queue.pop_front() {
        let du = dist[u.index()];
        for &w in graph.neighbors(u) {
            if dist[w.index()] == Dist::UNREACHABLE {
                dist[w.index()] = du + 1;
                queue.push_back(w);
            }
        }
    }
    dist
}
