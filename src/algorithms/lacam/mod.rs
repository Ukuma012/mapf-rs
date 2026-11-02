//! LaCAM: Lazy Constraints Addition search for MAPF.
//!
//! Keisuke Okumura, "LaCAM: Search-Based Algorithm for Quick Multi-Agent
//! Pathfinding", AAAI 2023, and "Improving LaCAM for Scalable Eventually
//! Optimal Multi-Agent Pathfinding", IJCAI 2023 (the anytime variant,
//! [`solve_star`]).
//!
//! Ported from <https://github.com/Kei18/lacam0> (MIT License,
//! Copyright (c) 2025 Keisuke Okumura).

mod pibt;

use std::collections::{BTreeSet, HashMap, VecDeque};

use pibt::Generator;

use crate::base::{AgentId, Config, Deadline, DistTable, Graph, Instance, Rng, Solution, VertexId};

/// Agent priority: how many timesteps it has spent away from its goal, then
/// its distance from the goal in the initial configuration.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Priority {
    elapsed: u32,
    init_d: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct HNodeId(usize);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct LNodeId(usize);

struct HNode {
    config: Config,
    parent: Option<HNodeId>,
    neighbors: BTreeSet<usize>,
    g: usize,
    h: usize,
    f: usize,
    priorities: Vec<Priority>,
    order: Vec<AgentId>,
    search_tree: VecDeque<LNodeId>,
}

/// `(agent, vertex)` pairs; the length is the depth in the low-level tree.
struct LNode {
    constraints: Vec<(AgentId, VertexId)>,
}

struct LaCAM<'a> {
    graph: &'a Graph,
    goals: &'a [VertexId],
    dist: DistTable<'a>,
    generator: Generator<'a>,
    rng: Rng,

    h_nodes: Vec<HNode>,
    l_nodes: Vec<LNode>,
    open: VecDeque<HNodeId>,
    explored: HashMap<Config, HNodeId>,
    h_goal: Option<HNodeId>,
    star: bool,
}

impl<'a> LaCAM<'a> {
    fn new(instance: &'a Instance, seed: u64, star: bool) -> Self {
        let graph = instance.graph();
        Self {
            graph,
            goals: instance.goals(),
            dist: DistTable::new_parallel(graph, instance.goals()),
            generator: Generator::new(graph, instance.goals(), seed),
            rng: Rng::new(seed),
            h_nodes: Vec::new(),
            l_nodes: Vec::new(),
            open: VecDeque::new(),
            explored: HashMap::new(),
            h_goal: None,
            star,
        }
    }

    fn h(&self, id: HNodeId) -> &HNode {
        &self.h_nodes[id.0]
    }

    fn h_mut(&mut self, id: HNodeId) -> &mut HNode {
        &mut self.h_nodes[id.0]
    }

    fn h_value(&mut self, config: &[VertexId]) -> usize {
        (0..config.len())
            .map(|i| self.dist.get(self.goals[i], config[i]).value().unwrap_or(0) as usize)
            .sum()
    }

    fn edge_cost(&self, from: &[VertexId], to: &[VertexId]) -> usize {
        (0..from.len())
            .filter(|&i| from[i] != self.goals[i] || to[i] != self.goals[i])
            .count()
    }

    fn create_h_node(&mut self, config: &Config, parent: Option<HNodeId>) -> HNodeId {
        let n = config.len();

        let g = match parent {
            None => 0,
            Some(p) => {
                let from = self.h(p).config.clone();
                self.h(p).g + self.edge_cost(&from, config)
            }
        };
        let h = self.h_value(config);

        let priorities: Vec<Priority> = match parent {
            None => (0..n)
                .map(|i| Priority {
                    elapsed: 0,
                    init_d: self.dist.get(self.goals[i], config[i]).value().unwrap_or(0),
                })
                .collect(),
            Some(p) => {
                let parent_p = self.h(p).priorities.clone();
                (0..n)
                    .map(|i| Priority {
                        elapsed: if config[i] == self.goals[i] {
                            0
                        } else {
                            parent_p[i].elapsed + 1
                        },
                        init_d: parent_p[i].init_d,
                    })
                    .collect()
            }
        };

        let mut order: Vec<AgentId> = (0..n as u32).map(AgentId).collect();
        order.sort_unstable_by(|a, b| priorities[b.index()].cmp(&priorities[a.index()]));

        self.l_nodes.push(LNode {
            constraints: Vec::new(),
        });
        let mut search_tree = VecDeque::new();
        search_tree.push_back(LNodeId(self.l_nodes.len() - 1));

        let id = HNodeId(self.h_nodes.len());
        self.h_nodes.push(HNode {
            config: config.clone(),
            parent,
            neighbors: BTreeSet::new(),
            g,
            h,
            f: g + h,
            priorities,
            order,
            search_tree,
        });

        if let Some(p) = parent {
            self.h_mut(id).neighbors.insert(p.0);
            self.h_mut(p).neighbors.insert(id.0);
        }
        self.explored.insert(config.clone(), id);
        id
    }

    fn next_l_node(&mut self, h_id: HNodeId) -> Option<LNodeId> {
        let l_id = self.h_mut(h_id).search_tree.pop_front()?;
        let depth = self.l_nodes[l_id.0].constraints.len();
        let n = self.h(h_id).config.len();

        if depth < n {
            let parent_constraints = self.l_nodes[l_id.0].constraints.clone();
            let a = self.h(h_id).order[depth];
            let v_now = self.h(h_id).config[a.index()];

            let mut cands: Vec<VertexId> = self.graph.actions(v_now).to_vec();
            self.rng.shuffle(&mut cands);

            for u in cands {
                let mut constraints = parent_constraints.clone();
                constraints.push((a, u));
                self.l_nodes.push(LNode { constraints });
                let child = LNodeId(self.l_nodes.len() - 1);
                self.h_mut(h_id).search_tree.push_back(child);
            }
        }
        Some(l_id)
    }

    fn solve(&mut self, starts: &[VertexId], deadline: &Deadline) -> Option<Solution> {
        for i in 0..starts.len() {
            if !self.dist.get(self.goals[i], starts[i]).is_reachable() {
                return None;
            }
        }

        let init = self.create_h_node(&starts.to_vec(), None);
        self.open.push_front(init);

        while let Some(&h_id) = self.open.front() {
            if deadline.expired() {
                break;
            }

            if let Some(goal) = self.h_goal {
                if self.h(h_id).f >= self.h(goal).f {
                    self.open.pop_front();
                    continue;
                }
            }

            if self.h_goal.is_none() && self.h(h_id).config == self.goals {
                self.h_goal = Some(h_id);
                if !self.star {
                    break;
                }
                continue;
            }

            let Some(l_id) = self.next_l_node(h_id) else {
                self.open.pop_front();
                continue;
            };

            let q_from = self.h(h_id).config.clone();
            let order = self.h(h_id).order.clone();
            let mut q_to: Vec<Option<VertexId>> = vec![None; q_from.len()];
            for &(a, v) in &self.l_nodes[l_id.0].constraints {
                q_to[a.index()] = Some(v);
            }

            if !self
                .generator
                .set_new_config(&q_from, &mut q_to, &order, &mut self.dist)
            {
                continue;
            }
            let q_to: Config = q_to.into_iter().map(|v| v.unwrap()).collect();

            match self.explored.get(&q_to).copied() {
                Some(existing) => {
                    self.rewrite(h_id, existing);
                    self.open.push_front(existing);
                }
                None => {
                    let new = self.create_h_node(&q_to, Some(h_id));
                    self.open.push_front(new);
                }
            }
        }

        self.h_goal.map(|id| self.backtrack(id))
    }

    fn backtrack(&self, id: HNodeId) -> Solution {
        let mut configs = Vec::new();
        let mut cur = Some(id);
        while let Some(i) = cur {
            configs.push(self.h(i).config.clone());
            cur = self.h(i).parent;
        }
        configs.reverse();
        Solution::from_configs(configs)
    }

    fn rewrite(&mut self, from: HNodeId, to: HNodeId) {
        self.h_mut(from).neighbors.insert(to.0);

        let mut queue = VecDeque::new();
        queue.push_back(from);

        while let Some(n_from) = queue.pop_front() {
            let g = self.h(n_from).g;
            let config = self.h(n_from).config.clone();
            let neighbors: Vec<usize> = self.h(n_from).neighbors.iter().copied().collect();

            for n in neighbors {
                let n_to = HNodeId(n);
                let new_g = g + self.edge_cost(&config, &self.h(n_to).config.clone());
                if new_g < self.h(n_to).g {
                    let h = self.h(n_to).h;
                    let node = self.h_mut(n_to);
                    node.g = new_g;
                    node.f = new_g + h;
                    node.parent = Some(n_from);
                    queue.push_back(n_to);

                    if let Some(goal) = self.h_goal {
                        if self.h(n_to).f < self.h(goal).f {
                            self.open.push_front(n_to);
                        }
                    }
                }
            }
        }
    }
}

/// Runs LaCAM and returns the first solution found.
pub fn solve(instance: &Instance, seed: u64, deadline: &Deadline) -> Option<Solution> {
    LaCAM::new(instance, seed, false).solve(instance.starts(), deadline)
}

/// Runs LaCAM*, which keeps improving the solution until the deadline.
pub fn solve_star(instance: &Instance, seed: u64, deadline: &Deadline) -> Option<Solution> {
    LaCAM::new(instance, seed, true).solve(instance.starts(), deadline)
}
