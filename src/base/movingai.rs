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

pub(super) struct ScenEntry {
    pub start: (usize, usize),
    pub goal: (usize, usize),
}

pub(super) fn read_scen_file(path: &Path) -> Result<Vec<ScenEntry>> {
    let text = std::fs::read_to_string(path)?;
    parse_scen(&text).ok_or_else(|| Error::ParseScen {
        path: path.to_path_buf(),
    })
}

fn parse_scen(text: &str) -> Option<Vec<ScenEntry>> {
    let mut entries = Vec::new();
    for line in text.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 9 {
            continue;
        }
        entries.push(ScenEntry {
            start: (f[4].parse().ok()?, f[5].parse().ok()?),
            goal: (f[6].parse().ok()?, f[7].parse().ok()?),
        });
    }
    Some(entries)
}

#[cfg(test)]
mod tests {
    use super::parse_map;

    #[test]
    fn parses_valid_map() {
        // with a type line; T is an obstacle
        let (w, h, passable) =
            parse_map("type octile\nheight 2\nwidth 3\nmap\n.T@\n...\n").unwrap();
        assert_eq!((w, h), (3, 2));
        assert_eq!(passable, vec![true, false, false, true, true, true]);

        // no type line, CRLF endings
        assert!(parse_map("height 1\r\nwidth 2\r\nmap\r\n..\r\n").is_some());
    }

    #[test]
    fn rejects_broken_input() {
        for text in [
            "height 2\nwidth 2\n..\n..",        // no "map" line
            "height 2\nmap\n..\n..",            // missing width
            "width 2\nmap\n..\n..",             // missing height
            "height 0\nwidth 2\nmap\n",         // zero dimension
            "height 2\nwidth 4\nmap\n....\n..", // row shorter than width
            "height 3\nwidth 2\nmap\n..\n..",   // fewer rows than height
            "height 1\nwidth 3\nmap\n.S.",      // unknown terrain character
        ] {
            assert!(parse_map(text).is_none(), "should reject: {text:?}");
        }
    }
}
