use std::{collections::HashSet, path::Path, sync::Arc};

use crate::base::{Error, Graph, Result, VertexId, movingai};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AgentId(pub u32);

impl AgentId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Debug)]
pub struct Instance {
    graph: Arc<Graph>,
    starts: Vec<VertexId>,
    goals: Vec<VertexId>,
}

impl Instance {
    pub fn new(
        graph: impl Into<Arc<Graph>>,
        starts: Vec<VertexId>,
        goals: Vec<VertexId>,
    ) -> Result<Self> {
        let graph = graph.into();
        if starts.len() != goals.len() {
            return Err(Error::InvalidInstance);
        }

        let n = graph.num_vertices();
        if starts.iter().chain(&goals).any(|v| v.index() >= n) {
            return Err(Error::InvalidInstance);
        }

        if has_duplicate(&starts) || has_duplicate(&goals) {
            return Err(Error::InvalidInstance);
        }

        Ok(Self {
            graph,
            starts,
            goals,
        })
    }

    pub fn from_scen_file(
        graph: impl Into<Arc<Graph>>,
        path: impl AsRef<Path>,
        num_agents: usize,
    ) -> Result<Self> {
        let graph = graph.into();
        let entries = movingai::read_scen_file(path.as_ref())?;

        if entries.len() < num_agents {
            return Err(Error::NotEnoughAgents {
                requested: num_agents,
                available: entries.len(),
            });
        }
        let mut starts = Vec::new();
        let mut goals = Vec::new();

        for e in &entries[..num_agents] {
            starts.push(
                graph
                    .vertex_at(e.start.0, e.start.1)
                    .ok_or(Error::InvalidInstance)?,
            );
            goals.push(
                graph
                    .vertex_at(e.goal.0, e.goal.1)
                    .ok_or(Error::InvalidInstance)?,
            );
        }

        Self::new(graph, starts, goals)
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn starts(&self) -> &[VertexId] {
        &self.starts
    }

    pub fn goals(&self) -> &[VertexId] {
        &self.goals
    }

    pub fn num_agents(&self) -> usize {
        self.starts.len()
    }
}

fn has_duplicate(vertices: &[VertexId]) -> bool {
    let mut seen = HashSet::with_capacity(vertices.len());
    !vertices.iter().all(|v| seen.insert(*v))
}

pub type Config = Vec<VertexId>;
