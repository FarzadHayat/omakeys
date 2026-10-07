//! Physical key geometry for a keyboard. v1: LoremIpsum36 only.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyPos {
    pub row: u8,
    pub col: u8,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoardProfile {
    pub name: &'static str,
    pub keys: Vec<KeyPos>,
    pub rows: u8,
    pub cols: u8,
    /// Highest layer index drawn in v1 (skip frogv 9–13).
    pub max_layer_inclusive: u8,
}

fn visual_x(col: u8) -> f64 {
    let c = col as f64;
    if col >= 5 {
        c + 1.5
    } else {
        c
    }
}

/// LoremIpsum36 geometry from `keyboard/penk/loremipsum36/keyboard.json` LAYOUT,
/// with VIA-style 1.5u split gap between halves.
pub fn loremipsum36() -> BoardProfile {
    const MATRIX: &[(u8, u8)] = &[
        (0, 0),
        (0, 1),
        (0, 2),
        (0, 3),
        (0, 4),
        (0, 5),
        (0, 6),
        (0, 7),
        (0, 8),
        (0, 9),
        (1, 0),
        (1, 1),
        (1, 2),
        (1, 3),
        (1, 4),
        (1, 5),
        (1, 6),
        (1, 7),
        (1, 8),
        (1, 9),
        (2, 0),
        (2, 1),
        (2, 2),
        (2, 3),
        (2, 4),
        (2, 5),
        (2, 6),
        (2, 7),
        (2, 8),
        (2, 9),
        (3, 2),
        (3, 3),
        (3, 4),
        (3, 5),
        (3, 6),
        (3, 7),
    ];

    let keys = MATRIX
        .iter()
        .map(|&(row, col)| KeyPos {
            row,
            col,
            x: visual_x(col),
            y: row as f64,
            w: 1.0,
            h: 1.0,
        })
        .collect();

    BoardProfile {
        name: "loremipsum36",
        keys,
        rows: 4,
        cols: 10,
        max_layer_inclusive: 8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loremipsum36_has_36_keys_and_split_gap() {
        let p = loremipsum36();
        assert_eq!(p.keys.len(), 36);
        assert_eq!(p.max_layer_inclusive, 8);
        let left = p.keys.iter().find(|k| k.row == 0 && k.col == 4).unwrap();
        let right = p.keys.iter().find(|k| k.row == 0 && k.col == 5).unwrap();
        assert!((right.x - left.x - left.w) >= 1.4);
    }
}
