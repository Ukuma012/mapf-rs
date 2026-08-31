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
    scan(paths.len(), len, |a, t| {
        *paths[a].get(t).unwrap_or_else(|| paths[a].last().unwrap())
    })
}

pub(super) fn scan(
    num_agents: usize,
    len: usize,
    pos: impl Fn(usize, usize) -> VertexId,
) -> Option<Conflict> {
    if num_agents == 0 || len == 0 {
        return None;
    }

    let mut nv = 0;
    for t in 0..len {
        for a in 0..num_agents {
            nv = nv.max(pos(a, t).index() + 1);
        }
    }

    let mut occ = vec![0u32; nv];
    let mut stamp = vec![0u32; nv];

    for t in 0..len {
        let mark = (t + 1) as u32;

        for a in 0..num_agents {
            let v = pos(a, t);
            let vi = v.index();
            if stamp[vi] == mark {
                return Some(Conflict::Vertex {
                    t,
                    a: AgentId(occ[vi]),
                    b: AgentId(a as u32),
                    v,
                });
            }
            stamp[vi] = mark;
            occ[vi] = a as u32;
        }

        if t + 1 == len {
            break;
        }

        for a in 0..num_agents {
            let from = pos(a, t);
            let to = pos(a, t + 1);
            if from == to {
                continue;
            }
            let ti = to.index();
            if stamp[ti] == mark {
                let b = occ[ti] as usize;
                if b != a && pos(b, t + 1) == from {
                    let (a, b) = (a.min(b), a.max(b));
                    return Some(Conflict::Swap {
                        t,
                        a: AgentId(a as u32),
                        b: AgentId(b as u32),
                        from: pos(a, t),
                        to: pos(a, t + 1),
                    });
                }
            }
        }
    }
    None
}
