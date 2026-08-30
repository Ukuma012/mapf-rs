use std::fmt;

use crate::base::{AgentId, VertexId, instance::Config};

pub struct Solution {
    configs: Vec<Config>,
}

impl Solution {
    pub fn from_configs(configs: Vec<Config>) -> Self {
        Self { configs }
    }

    pub fn from_paths(paths: Vec<Vec<VertexId>>) -> Self {
        assert!(
            paths.iter().all(|p| !p.is_empty()),
            "each path must contain at least the start vertex"
        );

        let len = paths.iter().map(|p| p.len()).max().unwrap_or(0);
        let mut configs = Vec::with_capacity(len);
        for t in 0..len {
            configs.push(
                paths
                    .iter()
                    .map(|p| *p.get(t).unwrap_or_else(|| p.last().unwrap()))
                    .collect(),
            );
        }
        Self { configs }
    }

    pub fn len(&self) -> usize {
        self.configs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.configs.is_empty()
    }

    pub fn makespan(&self) -> usize {
        self.configs.len().saturating_sub(1)
    }

    pub fn path_cost(&self, a: AgentId) -> usize {
        if self.configs.is_empty() {
            return 0;
        }
        let agent = a.index();
        let goal = self.configs[self.configs.len() - 1][agent];
        let mut cost = self.configs.len();
        while cost > 0 && self.configs[cost - 1][agent] == goal {
            cost -= 1;
        }
        cost
    }

    pub fn sum_of_costs(&self) -> usize {
        (0..self.num_agents())
            .map(|a| self.path_cost(AgentId(a as u32)))
            .sum()
    }

    pub fn sum_of_loss(&self) -> usize {
        if self.configs.is_empty() {
            return 0;
        }
        let last = self.configs.len() - 1;
        let mut total = 0;
        for agent in 0..self.num_agents() {
            let goal = self.configs[last][agent];
            for t in 1..self.configs.len() {
                if self.configs[t - 1][agent] != goal || self.configs[t][agent] != goal {
                    total += 1;
                }
            }
        }
        total
    }

    pub fn num_agents(&self) -> usize {
        self.configs.first().map_or(0, |c| c.len())
    }

    pub fn config_at(&self, t: usize) -> &[VertexId] {
        &self.configs[t]
    }

    pub fn configs(&self) -> &[Config] {
        &self.configs
    }

    pub fn path_of(&self, a: AgentId) -> impl Iterator<Item = VertexId> {
        self.configs.iter().map(move |c| c[a.index()])
    }
}

impl fmt::Debug for Solution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Solution")
            .field("makespan", &self.makespan())
            .field("num_agents", &self.num_agents())
            .finish()
    }
}
