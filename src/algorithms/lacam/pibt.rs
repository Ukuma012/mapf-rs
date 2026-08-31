//! PIBT used as LaCAM's configuration generator.
//!
//! Unlike [`crate::algorithms::pibt`] this variant starts from a partially
//! filled configuration (the constraints of a low-level node), takes the agent
//! order from the caller, and reports whether the constrained configuration
//! could be completed.
//!
//! Ported from <https://github.com/Kei18/lacam0> (MIT License,
//! Copyright (c) 2025 Keisuke Okumura).

use std::cmp::Reverse;

use crate::base::{AgentId, DistTable, Graph, Rng, VertexId};

const MAX_ACTIONS: usize = 5;

pub(super) struct Generator<'a> {
    graph: &'a Graph,
    goals: &'a [VertexId],
    occupied_now: Vec<Option<AgentId>>,
    occupied_next: Vec<Option<AgentId>>,
    rng: Rng,
}

impl<'a> Generator<'a> {
    pub(super) fn new(graph: &'a Graph, goals: &'a [VertexId], seed: u64) -> Self {
        let n = graph.num_vertices();
        Self {
            graph,
            goals,
            occupied_now: vec![None; n],
            occupied_next: vec![None; n],
            rng: Rng::new(seed),
        }
    }

    /// Completes `q_to`, which already holds the constrained agents.
    /// Returns false if the constraints cannot be satisfied.
    pub(super) fn set_new_config(
        &mut self,
        q_from: &[VertexId],
        q_to: &mut [Option<VertexId>],
        order: &[AgentId],
        dist: &mut DistTable,
    ) -> bool {
        let n = q_from.len();
        let mut success = true;

        for i in 0..n {
            self.occupied_now[q_from[i].index()] = Some(AgentId(i as u32));
            if let Some(v) = q_to[i] {
                if self.occupied_next[v.index()].is_some() {
                    success = false;
                    break;
                }
                if let Some(j) = self.occupied_now[v.index()] {
                    if j.index() != i && q_to[j.index()] == Some(q_from[i]) {
                        success = false;
                        break;
                    }
                }
                self.occupied_next[v.index()] = Some(AgentId(i as u32));
            }
        }

        if success {
            for &a in order {
                if q_to[a.index()].is_none() && !self.func_pibt(a, q_from, q_to, dist) {
                    success = false;
                    break;
                }
            }
        }

        for i in 0..n {
            self.occupied_now[q_from[i].index()] = None;
            if let Some(v) = q_to[i] {
                self.occupied_next[v.index()] = None;
            }
        }

        success
    }

    fn func_pibt(
        &mut self,
        ai: AgentId,
        q_from: &[VertexId],
        q_to: &mut [Option<VertexId>],
        dist: &mut DistTable,
    ) -> bool {
        let graph = self.graph;
        let v_now = q_from[ai.index()];
        let goal = self.goals[ai.index()];
        let actions = graph.actions(v_now);
        let k = actions.len();

        let mut cand = [VertexId(0); MAX_ACTIONS];
        cand[..k].copy_from_slice(actions);
        self.rng.shuffle(&mut cand[..k]);
        cand[..k].sort_by_key(|&v| dist.get(goal, v));

        let swap_agent = self.swap_partner(ai, cand[0], q_from, q_to, dist);
        if swap_agent.is_some() {
            self.rng.shuffle(&mut cand[..k]);
            cand[..k].sort_by_key(|&v| Reverse(dist.get(goal, v)));
        }

        for i in 0..k {
            let u = cand[i];
            if self.occupied_next[u.index()].is_some() {
                continue;
            }
            if let Some(aj) = self.occupied_now[u.index()] {
                if q_to[aj.index()] == Some(v_now) {
                    continue;
                }
            }

            self.occupied_next[u.index()] = Some(ai);
            q_to[ai.index()] = Some(u);

            if let Some(aj) = self.occupied_now[u.index()] {
                if u != v_now
                    && q_to[aj.index()].is_none()
                    && !self.func_pibt(aj, q_from, q_to, dist)
                {
                    continue;
                }
            }

            if i == 0 {
                if let Some(sa) = swap_agent {
                    if q_to[sa.index()].is_none() && self.occupied_next[v_now.index()].is_none() {
                        self.occupied_next[v_now.index()] = Some(sa);
                        q_to[sa.index()] = Some(v_now);
                    }
                }
            }
            return true;
        }

        self.occupied_next[v_now.index()] = Some(ai);
        q_to[ai.index()] = Some(v_now);
        false
    }

    fn swap_partner(
        &mut self,
        ai: AgentId,
        target: VertexId,
        q_from: &[VertexId],
        q_to: &[Option<VertexId>],
        dist: &mut DistTable,
    ) -> Option<AgentId> {
        let v_now = q_from[ai.index()];
        if target == v_now {
            return None;
        }

        if let Some(aj) = self.occupied_now[target.index()] {
            let v_j = q_from[aj.index()];
            if aj != ai
                && q_to[aj.index()].is_none()
                && self.is_swap_required(ai, aj, v_now, v_j, dist)
                && self.is_swap_possible(v_j, v_now)
            {
                return Some(aj);
            }
        }

        for idx in 0..self.graph.neighbors(v_now).len() {
            let u = self.graph.neighbors(v_now)[idx];
            if let Some(ak) = self.occupied_now[u.index()] {
                if ak != ai
                    && target != q_from[ak.index()]
                    && self.is_swap_required(ak, ai, v_now, target, dist)
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
        dist: &mut DistTable,
    ) -> bool {
        let graph = self.graph;
        let g_pusher = self.goals[pusher.index()];
        let g_puller = self.goals[puller.index()];

        let mut v_pusher = v_pusher_origin;
        let mut v_puller = v_puller_origin;
        let mut tmp = None;

        while dist.get(g_pusher, v_puller) < dist.get(g_pusher, v_pusher) {
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

        let puller_prefers_pusher = dist.get(g_puller, v_pusher) < dist.get(g_puller, v_puller);
        let pusher_at_goal_or_benefits =
            v_pusher == g_pusher || dist.get(g_pusher, v_puller) < dist.get(g_pusher, v_pusher);

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
