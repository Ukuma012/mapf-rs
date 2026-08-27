use crate::base::{Error, Result};
use std::path::Path;

pub(super) fn read_map_file(path: &Path) -> Result<(usize, usize, Vec<bool>)> {
    let text = std::fs::read_to_string(path)?;
    parse_map(&text).ok_or_else(|| Error::ParseMap {
        path: path.to_path_buf(),
    })
}

fn parse_map(text: &str) -> Option<(usize, usize, Vec<bool>)> {
    let mut lines = text.lines();
    let (mut width, mut height) = (None, None);

    loop {
        let line = lines.next()?;
        let mut it = line.split_whitespace();
        let key = it.next();
        match key {
            Some("map") => break,
            Some("height") | Some("width") => {
                if let Some(n) = it
                    .next()
                    .and_then(|s| s.parse::<usize>().ok())
                    .filter(|&n| n > 0)
                {
                    if key == Some("height") {
                        height = Some(n);
                    } else {
                        width = Some(n);
                    }
                }
            }
            _ => {} 
        }
    }

    let width = width?;
    let height = height?;

    let mut passable = Vec::with_capacity(width * height);
    for _ in 0..height {
        let line = lines.next()?;
        if line.chars().count() < width {
            return None;
        }
        for c in line.chars().take(width) {
            passable.push(match c {
                '.' => true,
                '@' | 'T' => false,
                _ => return None,
            });
        }
    }

    Some((width, height, passable))
}
