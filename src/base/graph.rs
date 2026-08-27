use crate::base::Result;
use std::{
    fmt::{self, Write},
    path::Path,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VertexId(pub u32);

impl VertexId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Vertex {
    x: u16,
    y: u16,
    actions: Vec<VertexId>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Graph {
    vertices: Vec<Vertex>,
    grid: Vec<Option<VertexId>>,
    width: usize,
    height: usize,
}

impl fmt::Debug for Graph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Graph")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("num_vertices", &self.vertices.len())
            .finish()
    }
}

impl fmt::Display for Graph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            if y > 0 {
                f.write_char('\n')?;
            }
            for x in 0..self.width {
                let c = if self.vertex_at(x, y).is_some() {
                    '.'
                } else {
                    '@'
                };
                f.write_char(c)?;
            }
        }
        Ok(())
    }
}

impl Graph {
    pub fn actions(&self, v: VertexId) -> &[VertexId] {
        &self.vertices[v.index()].actions
    }

    pub fn neighbors(&self, v: VertexId) -> &[VertexId] {
        let a = &self.vertices[v.index()].actions;
        &a[..a.len() - 1]
    }

    pub fn coord(&self, v: VertexId) -> (u16, u16) {
        let vertex = &self.vertices[v.index()];
        (vertex.x, vertex.y)
    }

    pub fn vertex_at(&self, x: usize, y: usize) -> Option<VertexId> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.grid[y * self.width + x]
    }

    pub fn num_vertices(&self) -> usize {
        self.vertices.len()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn from_grid(width: usize, height: usize, passable: &[bool]) -> Self {
        assert_eq!(
            passable.len(),
            width * height,
            "passable.len() must equal width * height"
        );

        let mut grid = vec![None; width * height];
        let mut vertices = Vec::new();
        for y in 0..height {
            for x in 0..width {
                if passable[y * width + x] {
                    grid[y * width + x] = Some(VertexId(vertices.len() as u32));
                    vertices.push(Vertex {
                        x: x as u16,
                        y: y as u16,
                        actions: Vec::new(),
                    })
                }
            }
        }

        for i in 0..vertices.len() {
            let (x, y) = (vertices[i].x as isize, vertices[i].y as isize);
            let mut actions = Vec::with_capacity(5);
            for (dx, dy) in [(-1, 0), (1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
                    continue;
                }
                if let Some(n) = grid[ny as usize * width + nx as usize] {
                    actions.push(n);
                }
            }
            actions.push(VertexId(i as u32));
            vertices[i].actions = actions
        }

        Self {
            vertices,
            grid,
            width,
            height,
        }
    }

    pub fn from_movingai_file(path: impl AsRef<Path>) -> Result<Self> {
        let (width, height, passable) = super::movingai::read_map_file(path.as_ref())?;
        Ok(Self::from_grid(width, height, &passable))
    }
}
