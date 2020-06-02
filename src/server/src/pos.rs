use crate::tile::Position;
use serde::{Deserialize, Serialize};

const VERTICE_COUPLE: [Position; 2] = [Position { x: 1, y: 0 }, Position { x: 0, y: -1 }];

#[derive(Debug, Serialize, Deserialize)]
pub struct PosGenerator {
    step: u32,
    last_pos: Position,
    tiles: Vec<Position>,
    vertices: Vec<Position>,
}

impl PosGenerator {
    pub fn new(step: u32) -> PosGenerator {
        PosGenerator {
            step: step,
            tiles: Vec::new(),
            vertices: Vec::new(),
            last_pos: Position { x: 0, y: 0 },
        }
    }
    fn total_pos(&self) -> u32 {
        let mut nbr = 1;
        for i in 0..self.step {
            nbr += 2 * i;
        }
        nbr
    }

    fn next_vertices(&mut self) -> () {
        let mut sign: i32 = 1;
        for _ in 0..2 {
            for i in 0..2 {
                self.vertices.push(Position {
                    x: VERTICE_COUPLE[i].x * sign * self.step as i32,
                    y: VERTICE_COUPLE[i].y * sign * self.step as i32,
                });
            }
            sign *= -1;
        }
    }

    fn next_tiles(&mut self) -> () {
        if self.vertices.len() == 0 {
            self.next_vertices();
        }
        let Position { x, y } = self.vertices.remove(0);
        for i in 0..(x | y) {
            self.tiles.push(Position {
                x: (self.last_pos.x | self.last_pos.x + i),
                y: (self.last_pos.y | self.last_pos.y + i),
            })
        }
    }
}

impl Iterator for PosGenerator {
    type Item = Position;
    fn next(&mut self) -> Option<Self::Item> {
        if self.tiles.len() == 0 {
            self.next_tiles();
        }
        Some(self.tiles.remove(0))
    }
}
