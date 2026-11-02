//! PIBT: Priority Inheritance with Backtracking.
//!
//! Okumura et al., "Priority Inheritance with Backtracking for Iterative
//! Multi-agent Path Finding", IJCAI 2019.
//!
//! The swap operation and the priority-inheritance procedure are ported from
//! <https://github.com/Kei18/lacam0> (MIT License, Copyright (c) 2025 Keisuke
//! Okumura).

use std::cmp::{Ordering, Reverse};
use std::mem;

use crate::base::{
    AgentId, Config, Deadline, Dist, DistTable, Graph, Instance, Rng, Solution, VertexId,
};

const MAX_ACTIONS: usize = 5;

struct Agent {
    v_now: VertexId,
    v_next: Option<VertexId>,
    elapsed: u32,
    init_d: Dist,
    tie_breaker: u64,
}

fn priority(a: &Agent, b: &Agent) -> Ordering {
    b.elapsed
        .cmp(&a.elapsed)
        .then(b.init_d.cmp(&a.init_d))
        .then(b.tie_breaker.cmp(&a.tie_breaker))
}

pub struct Pibt<'a> {
    graph: &'a Graph,
    goals: &'a [VertexId],
    dist: DistTable<'a>,
    agents: Vec<Agent>,
    order: Vec<AgentId>,
    occupied_now: Vec<Option<AgentId>>,
    occupied_next: Vec<Option<AgentId>>,
    rng: Rng,
}

impl<'a> Pibt<'a> {
    pub fn new(graph: &'a Graph, starts: &[VertexId], goals: &'a [VertexId], seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let mut dist = DistTable::new_parallel(graph, goals);
        let n = graph.num_vertices();

        let agents: Vec<Agent> = starts
            .iter()
            .enumerate()
            .map(|(i, &v)| Agent {
                v_now: v,
                v_next: None,
                elapsed: 0,
                init_d: dist.get(goals[i], v),
                tie_breaker: rng.next_u64(),
            })
            .collect();

        let mut occupied_now = vec![None; n];
        for (i, a) in agents.iter().enumerate() {
            occupied_now[a.v_now.index()] = Some(AgentId(i as u32));
        }
        let order = (0..agents.len() as u32).map(AgentId).collect();

        Self {
            graph,
            goals,
            dist,
            agents,
            order,
            occupied_now,
            occupied_next: vec![None; n],
            rng,
        }
    }

    pub fn run(&mut self, deadline: &Deadline) -> Option<Solution> {
        if self.agents.iter().any(|a| !a.init_d.is_reachable()) {
            return None;
        }

        let mut configs: Vec<Config> = vec![self.agents.iter().map(|a| a.v_now).collect()];

        while !deadline.expired() {
            if self.all_at_goal() {
                return Some(Solution::from_configs(mem::take(&mut configs)));
            }
            configs.push(self.step());
        }

        None
    }

    fn all_at_goal(&self) -> bool {
        self.agents
            .iter()
            .zip(self.goals)
            .all(|(a, &g)| a.v_now == g)
    }

    fn step(&mut self) -> Config {
        let mut order = mem::take(&mut self.order);
        {
            let agents = &self.agents;
            order.sort_unstable_by(|&x, &y| priority(&agents[x.index()], &agents[y.index()]));
        }

        for i in 0..order.len() {
            let a = order[i];
            if self.agents[a.index()].v_next.is_none() {
                self.func_pibt(a, None);
            }
        }
        self.order = order;

        let mut config = Vec::with_capacity(self.agents.len());
        for i in 0..self.agents.len() {
            let id = AgentId(i as u32);
            let v_now = self.agents[i].v_now;
            let v_next = self.agents[i].v_next.take().expect("v_next must be set");

            if self.occupied_now[v_now.index()] == Some(id) {
                self.occupied_now[v_now.index()] = None;
            }
            self.occupied_next[v_next.index()] = None;
            self.occupied_now[v_next.index()] = Some(id);

            self.agents[i].elapsed = if v_next == self.goals[i] {
                0
            } else {
                self.agents[i].elapsed + 1
            };
            self.agents[i].v_now = v_next;
            config.push(v_next);
        }
        config
    }

    fn func_pibt(&mut self, ai: AgentId, aj: Option<AgentId>) -> bool {
        let graph = self.graph;
        let v_now = self.agents[ai.index()].v_now;
        let goal = self.goals[ai.index()];
        let actions = graph.actions(v_now);
        let k = actions.len();

        let mut cand = [VertexId(0); MAX_ACTIONS];
        cand[..k].copy_from_slice(actions);
        self.rng.shuffle(&mut cand[..k]);
        {
            let dist = &mut self.dist;
            cand[..k].sort_by_key(|&v| dist.get(goal, v));
        }

        let swap_agent = self.swap_partner(ai, cand[0]);
        if swap_agent.is_some() {
            self.rng.shuffle(&mut cand[..k]);
            let dist = &mut self.dist;
            cand[..k].sort_by_key(|&v| Reverse(dist.get(goal, v)));
        }

        for i in 0..k {
            let u = cand[i];
            if self.occupied_next[u.index()].is_some() {
                continue;
            }
            if let Some(aj) = aj {
                if self.agents[aj.index()].v_now == u {
                    continue;
                }
            }

            self.occupied_next[u.index()] = Some(ai);
            self.agents[ai.index()].v_next = Some(u);

            if let Some(ak) = self.occupied_now[u.index()] {
                if ak != ai
                    && self.agents[ak.index()].v_next.is_none()
                    && !self.func_pibt(ak, Some(ai))
                {
                    continue;
                }
            }

            if i == 0 {
                if let Some(sa) = swap_agent {
                    if self.agents[sa.index()].v_next.is_none()
                        && self.occupied_next[v_now.index()].is_none()
                    {
                        self.occupied_next[v_now.index()] = Some(sa);
                        self.agents[sa.index()].v_next = Some(v_now);
                    }
                }
            }
            return true;
        }

        self.occupied_next[v_now.index()] = Some(ai);
        self.agents[ai.index()].v_next = Some(v_now);
        false
    }

    fn swap_partner(&mut self, ai: AgentId, target: VertexId) -> Option<AgentId> {
        let v_now = self.agents[ai.index()].v_now;
        if target == v_now {
            return None;
        }

        if let Some(aj) = self.occupied_now[target.index()] {
            let v_j = self.agents[aj.index()].v_now;
            if aj != ai
                && self.agents[aj.index()].v_next.is_none()
                && self.is_swap_required(ai, aj, v_now, v_j)
                && self.is_swap_possible(v_j, v_now)
            {
                return Some(aj);
            }
        }

        for idx in 0..self.graph.neighbors(v_now).len() {
            let u = self.graph.neighbors(v_now)[idx];
            if let Some(ak) = self.occupied_now[u.index()] {
                if ak != ai
                    && target != self.agents[ak.index()].v_now
                    && self.is_swap_required(ak, ai, v_now, target)
                    && self.is_swap_possible(target, v_now)
                {
                    return Some(ak);
                }
            }
        }

        None
    }

    fn is_swap_required(
        &mut self,
        pusher: AgentId,
        puller: AgentId,
        v_pusher_origin: VertexId,
        v_puller_origin: VertexId,
    ) -> bool {
        let graph = self.graph;
        let g_pusher = self.goals[pusher.index()];
        let g_puller = self.goals[puller.index()];

        let mut v_pusher = v_pusher_origin;
        let mut v_puller = v_puller_origin;
        let mut tmp = None;

        while self.dist.get(g_pusher, v_puller) < self.dist.get(g_pusher, v_pusher) {
            let neighbors = graph.neighbors(v_puller);
            let mut n = neighbors.len();
            for &u in neighbors {
                if u == v_pusher || self.is_dead_end_goal(u) {
                    n -= 1;
                } else {
                    tmp = Some(u);
                }
            }
            if n >= 2 {
                return false;
            }
            if n == 0 {
                break;
            }
            v_pusher = v_puller;
            v_puller = tmp.unwrap();
        }

        let puller_prefers_pusher =
            self.dist.get(g_puller, v_pusher) < self.dist.get(g_puller, v_puller);
        let pusher_at_goal_or_benefits = v_pusher == g_pusher
            || self.dist.get(g_pusher, v_puller) < self.dist.get(g_pusher, v_pusher);

        puller_prefers_pusher && pusher_at_goal_or_benefits
    }

    fn is_swap_possible(&self, v_pusher_origin: VertexId, v_puller_origin: VertexId) -> bool {
        let graph = self.graph;
        let mut v_pusher = v_pusher_origin;
        let mut v_puller = v_puller_origin;
        let mut tmp = None;

        while v_puller != v_pusher_origin {
            let neighbors = graph.neighbors(v_puller);
            let mut n = neighbors.len();
            for &u in neighbors {
                if u == v_pusher || self.is_dead_end_goal(u) {
                    n -= 1;
                } else {
                    tmp = Some(u);
                }
            }
            if n >= 2 {
                return true;
            }
            if n == 0 {
                return false;
            }
            v_pusher = v_puller;
            v_puller = tmp.unwrap();
        }

        false
    }

    fn is_dead_end_goal(&self, v: VertexId) -> bool {
        self.graph.neighbors(v).len() == 1
            && self.occupied_now[v.index()].is_some_and(|a| self.goals[a.index()] == v)
    }
}

pub fn solve(ins: &Instance, seed: u64, deadline: &Deadline) -> Option<Solution> {
    let mut pibt = Pibt::new(ins.graph(), ins.starts(), ins.goals(), seed);
    pibt.run(deadline)
}
