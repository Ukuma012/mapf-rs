use std::fmt;
use std::io::{self, Write};
use std::path::Path;

use crate::base::conflict::scan;
use crate::base::{AgentId, Conflict, Graph, Instance, Result, VertexId, instance::Config};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    Empty,
    NumAgentsMismatch {
        expected: usize,
        found: usize,
    },
    WrongStart {
        a: AgentId,
    },
    UnreachedGoal {
        a: AgentId,
    },
    IllegalMove {
        t: usize,
        a: AgentId,
        from: VertexId,
        to: VertexId,
    },
    Conflict(Conflict),
}

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

    /// Writes the solution in the text format read by
    /// [mapf-visualizer](https://github.com/Kei18/mapf-visualizer).
    pub fn write_visualizer<W: io::Write>(&self, w: &mut W, graph: &Graph) -> io::Result<()> {
        writeln!(w, "agents={}", self.num_agents())?;
        writeln!(w, "makespan={}", self.makespan())?;
        writeln!(w, "soc={}", self.sum_of_costs())?;
        writeln!(w, "solution=")?;
        for (t, config) in self.configs.iter().enumerate() {
            write!(w, "{t}:")?;
            for &v in config {
                let (x, y) = graph.coord(v);
                // the trailing comma is required by the visualizer's parser
                write!(w, "({x},{y}),")?;
            }
            writeln!(w)?;
        }
        Ok(())
    }

    /// Writes the solution to `path` in mapf-visualizer format.
    pub fn write_visualizer_file(&self, path: impl AsRef<Path>, graph: &Graph) -> Result<()> {
        let mut w = io::BufWriter::new(std::fs::File::create(path)?);
        self.write_visualizer(&mut w, graph)?;
        w.flush()?;
        Ok(())
    }

    pub fn validate(&self, ins: &Instance) -> Result<(), ValidationError> {
        if self.configs.is_empty() {
            return Err(ValidationError::Empty);
        }
        if self.num_agents() != ins.num_agents() {
            return Err(ValidationError::NumAgentsMismatch {
                expected: ins.num_agents(),
                found: self.num_agents(),
            });
        }

        let first = &self.configs[0];
        let last = &self.configs[self.configs.len() - 1];
        for a in 0..ins.num_agents() {
            if first[a] != ins.starts()[a] {
                return Err(ValidationError::WrongStart { a: AgentId(a as u32) });
            }
            if last[a] != ins.goals()[a] {
                return Err(ValidationError::UnreachedGoal { a: AgentId(a as u32) });
            }
        }

        let graph = ins.graph();
        for t in 1..self.configs.len() {
            for a in 0..ins.num_agents() {
                let (from, to) = (self.configs[t - 1][a], self.configs[t][a]);
                if !graph.actions(from).contains(&to) {
                    return Err(ValidationError::IllegalMove {
                        t,
                        a: AgentId(a as u32),
                        from,
                        to,
                    });
                }
            }
        }

        match scan(self.num_agents(), self.configs.len(), |a, t| {
            self.configs[t][a]
        }) {
            Some(c) => Err(ValidationError::Conflict(c)),
            None => Ok(()),
        }
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
