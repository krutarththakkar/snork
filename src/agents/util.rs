use std::cmp::Reverse;
use std::collections::VecDeque;
use std::f64;
use std::ops::Index;
use std::ops::IndexMut;
use std::usize;

use crate::env::*;

pub const BOARD_FREE: i8 = -1;
pub const BOARD_OBSTACLE: i8 = -2;
pub const BOARD_TMP: i8 = -3;
pub const BOARD_FOOD: i8 = -4;

const BOARD_SIZE: usize = 11;

#[derive(Clone)]
pub struct Grid {
    cells: [[i8; BOARD_SIZE]; BOARD_SIZE],
}

impl Grid {
    pub fn new() -> Grid {
        Grid {
            cells: [[-1; BOARD_SIZE]; BOARD_SIZE],
        }
    }

    pub fn filled(snakes: &[CSnake]) -> Grid {
        let mut board = Grid::new();

        for snake in snakes {
            for &p in &snake.body {
                board[p] = BOARD_OBSTACLE;
            }
        }

        board
    }

    pub fn populate(&mut self, food: &[Vec2D]) {
        for &p in food {
            if self.has(p) {
                self[p] = BOARD_FOOD;
            }
        }
    }

    pub fn avaliable(&self, p: Vec2D) -> bool {
        self.has(p) && self[p] != BOARD_OBSTACLE
    }

    pub fn has(&self, p: Vec2D) -> bool {
        0 <= p.x && p.x < BOARD_SIZE as _ && 0 <= p.y && p.y < BOARD_SIZE as _
    }

    pub fn count(&self, v: i8) -> usize {
        self.cells
            .iter()
            .map(|r| r.iter().filter(|&&c| c == v).count())
            .sum()
    }

    pub fn flood_fill(&mut self, heads: impl Iterator<Item = (i8, Vec2D)>) {
        let mut queue = VecDeque::with_capacity(BOARD_SIZE * BOARD_SIZE * 2);
        for (i, p) in heads {
            if self.avaliable(p) {
                queue.push_back((i, p));
            }
        }
        while let Some((i, p)) = queue.pop_front() {
            if self.has(p) && self[p] == BOARD_FREE {
                self[p] = i;

                for dir in Direction::iter() {
                    let p = p.apply(dir);
                    if self.has(p) && self[p] == BOARD_FREE {
                        queue.push_back((i, p));
                    }
                }
            }
        }
    }

    pub fn a_star(
        &self,
        start: Vec2D,
        target: Vec2D,
        first_move_heuristic: [f64; 4],
    ) -> Option<Vec<Vec2D>> {
        use priority_queue::PriorityQueue;
        use std::collections::HashMap;

        fn make_path(data: &HashMap<Vec2D, (Vec2D, f64)>, target: Vec2D) -> Vec<Vec2D> {
            let mut path = Vec::new();
            let mut p = target;
            while p.x >= 0 {
                path.push(p);
                p = data.get(&p).unwrap().0;
            }
            path.reverse();
            path
        }

        let mut queue = PriorityQueue::new();
        let mut data: HashMap<Vec2D, (Vec2D, f64)> = HashMap::new();
        data.insert(start, (Vec2D::new(-1, -1), 0.0));

        queue.push(start, Reverse(0));
        while let Some((front, _)) = queue.pop() {
            let cost = data.get(&front).unwrap().1;

            if front == target {
                return Some(make_path(&data, target));
            }

            for d in Direction::iter() {
                let neighbor = front.apply(d);
                let neighbor_cost = if front == start {
                    cost + 1.0 + first_move_heuristic[d as usize]
                } else {
                    cost + 1.0
                };

                if self.avaliable(neighbor) {
                    let cost_so_far = data.get(&neighbor).map(|(_, c)| *c).unwrap_or(f64::MAX);
                    if neighbor_cost < cost_so_far {
                        data.insert(neighbor, (front, neighbor_cost));
                        // queue does not accept float
                        let estimated_cost = neighbor_cost + (neighbor - start).manhattan() as f64;
                        queue.push(neighbor, Reverse((estimated_cost * 10.0) as usize));
                    }
                }
            }
        }

        None
    }

    pub fn space_after_move(&self, you_i: i8, snakes: &[CSnake]) -> [usize; 4] {
        let you = &snakes[you_i as usize];
        let snakes: Vec<(i8, &CSnake)> = snakes
            .iter()
            .enumerate()
            .map(|(i, s)| (i as i8, s))
            .collect();

        // Collect snake heads
        let mut next_heads: Vec<(i8, Vec2D)> = Vec::new();
        let longer_enemies = snakes
            .iter()
            .filter(|&(i, s)| *i != you_i && s.body.len() >= you.body.len());
        for &(i, s) in longer_enemies {
            // Expand longer snakes in all directions
            next_heads.extend(
                Direction::iter()
                    .map(|d| (i, s.head().apply(d)))
                    .filter(|(_, s)| self.avaliable(*s)),
            );
        }
        next_heads.push((you_i, you.head()));
        let shorter_enemies = snakes
            .iter()
            .filter(|&(i, s)| *i != you_i && s.body.len() < you.body.len());
        next_heads.extend(shorter_enemies.map(|(i, s)| (*i, s.head())));

        let mut space_after_move = [0; 4];
        for (dir_i, dir) in Direction::iter().enumerate() {
            let p = you.head().apply(dir);
            if self.avaliable(p) {
                let mut next_grid = self.clone();
                // mark head
                next_grid[p] = BOARD_OBSTACLE;
                // free tail
                for (_, snake) in &snakes {
                    next_grid[snake.body[snake.body.len() - 1]] = BOARD_FREE;
                }
                for &(_, p) in &next_heads {
                    next_grid[p] = BOARD_OBSTACLE;
                }

                next_grid.flood_fill(
                    next_heads
                        .iter()
                        .map(|&(i, s)| {
                            if i == you_i {
                                (i, s.apply(dir))
                            } else {
                                (i, s)
                            }
                        })
                        .flat_map(|(i, s)| Direction::iter().map(move |d| (i, s.apply(d)))),
                );
                space_after_move[dir_i] = next_grid.count(you_i);
            }
        }
        space_after_move
    }
}

impl Index<Vec2D> for Grid {
    type Output = i8;

    fn index(&self, p: Vec2D) -> &Self::Output {
        &self.cells[p.y as usize][p.x as usize]
    }
}

impl IndexMut<Vec2D> for Grid {
    fn index_mut(&mut self, p: Vec2D) -> &mut Self::Output {
        &mut self.cells[p.y as usize][p.x as usize]
    }
}

impl std::fmt::Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Grid {{")?;
        for row in self.cells.iter().rev() {
            write!(f, "  ")?;
            for cell in row {
                write!(f, "{:>3},", cell)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

pub struct CSnake {
    pub id: i8,
    pub body: Vec<Vec2D>,
}

impl CSnake {
    pub fn new(id: i8, body: Vec<Vec2D>) -> CSnake {
        CSnake { id, body }
    }

    pub fn head(&self) -> Vec2D {
        self.body[0]
    }
}

impl PartialEq for CSnake {
    fn eq(&self, s: &CSnake) -> bool {
        self.id == s.id
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_a_star() {
        use super::*;
        let grid = Grid::new();

        let path = grid.a_star(Vec2D::new(0, 0), Vec2D::new(1, 1), [1.0, 0.0, 0.0, 0.0]).unwrap();
        println!("{:?}", path);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], Vec2D::new(0, 0));
        assert_eq!(path[2], Vec2D::new(1, 1));
    }

    #[test]
    fn test_flood_fill() {
        use super::*;
        let mut grid = Grid::new();
        grid.flood_fill([(0, Vec2D::new(0, 0))].iter().cloned());
        println!("Filled {:?}", grid);

        let mut grid = Grid::new();
        grid.flood_fill(
            [(0, Vec2D::new(0, 0)), (1, Vec2D::new(10, 10))]
                .iter()
                .cloned(),
        );
        println!("Filled {:?}", grid);

        let mut grid = Grid::new();
        grid.flood_fill(
            [
                (0, Vec2D::new(0, 0)),
                (1, Vec2D::new(10, 10)),
                (2, Vec2D::new(5, 5)),
            ]
            .iter()
            .cloned(),
        );
        println!("Filled {:?}", grid);
    }

    #[test]
    fn test_space_after_move() {
        use super::*;
        let snakes = [CSnake::new(0, vec![Vec2D::new(0, 0)])];
        let grid = Grid::filled(&snakes);
        let space = grid.space_after_move(0, &snakes);
        println!("space after move {:?}", space);
    }
}
