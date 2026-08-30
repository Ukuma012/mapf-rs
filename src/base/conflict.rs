use crate::base::{AgentId, VertexId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conflict {
    Vertex {
        t: usize,
        a: AgentId,
        b: AgentId,
        v: VertexId,
    },
    Swap {
        t: usize,
        a: AgentId,
        b: AgentId,
        from: VertexId,
        to: VertexId,
    },
}

pub fn find_conflict_in_paths(paths: &[Vec<VertexId>]) -> Option<Conflict> {
    assert!(paths.iter().all(|p| !p.is_empty()));

    let len = paths.iter().map(|p| p.len()).max()?;
    let pos = |a: usize, t: usize| *paths[a].get(t).unwrap_or_else(|| paths[a].last().unwrap());

    for t in 0..len {
        for i in 0..paths.len() {
            for j in (i + 1)..paths.len() {
                let (pi, pj) = (pos(i, t), pos(j, t));
                if pi == pj {
                    return Some(Conflict::Vertex {
                        t,
                        a: AgentId(i as u32),
                        b: AgentId(j as u32),
                        v: pi,
                    });
                }
                if t + 1 < len {
                    let (ni, nj) = (pos(i, t + 1), pos(j, t + 1));
                    if ni == pj && nj == pi {
                        return Some(Conflict::Swap {
                            t,
                            a: AgentId(i as u32),
                            b: AgentId(j as u32),
                            from: pi,
                            to: ni,
                        });
                    }
                }
            }
        }
    }
    None
}
