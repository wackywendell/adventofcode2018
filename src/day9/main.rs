#![warn(clippy::all)]

use std::collections::VecDeque;

struct Game {
    marbles: VecDeque<i64>,
    marble: i64,
    scores: Vec<i64>,
}

impl Game {
    fn new(players: usize) -> Game {
        let mut ms = VecDeque::new();
        ms.push_back(0);
        Game {
            marbles: ms,
            marble: 1,
            scores: vec![0; players],
        }
    }

    fn rotate(&mut self, dist: isize) {
        if self.marbles.len() < 2 {
            return;
        }
        for _ in 0..dist {
            let m = self.marbles.pop_back().unwrap();
            self.marbles.push_front(m);
        }

        for _ in 0..-dist {
            let m = self.marbles.pop_front().unwrap();
            self.marbles.push_back(m);
        }
    }

    fn next(&mut self) {
        if self.marble % 23 == 0 {
            self.rotate(-7);
            let removed = self.marbles.pop_back().unwrap();
            let player = (self.marble as usize) % (self.scores.len());

            self.scores[player] += self.marble + removed;
            self.marble += 1;
            return;
        }

        self.rotate(2);
        self.marbles.push_back(self.marble);
        self.marble += 1;
    }

    fn play(&mut self, rounds: usize) {
        for _ in 0..rounds {
            self.next();
        }
    }

    fn high_score(&self) -> i64 {
        match self.scores.iter().max() {
            Some(&n) => n,
            None => 0,
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut g = Game::new(459);
    g.play(71320);
    println!("High score: {:?}", g.high_score());

    let mut g = Game::new(459);
    g.play(7_132_000);
    println!("High score after 100x more rounds: {:?}", g.high_score());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game() {
        let mut g = Game::new(9);
        g.play(25);

        // Below is the order from the site.
        // Actual order in the data structure is reversed with the "active" marble
        // at the end.
        // assert_eq!(
        //     g.marbles,
        //     vec!(
        //         0, 16, 8, 17, 4, 18, 19, 2, 24, 20, 25, 10, 21, 5, 22, 11, 1, 12, 6, 13, 3, 14, 7,
        //         15
        //     )
        // );
        assert_eq!(g.high_score(), 32);
    }

    #[test]
    fn test_high_scores() {
        let mut g = Game::new(10);
        g.play(1618);
        assert_eq!(g.high_score(), 8317);

        g = Game::new(13);
        g.play(7999);
        assert_eq!(g.high_score(), 146_373);

        g = Game::new(17);
        g.play(1104);
        assert_eq!(g.high_score(), 2764);

        g = Game::new(21);
        g.play(6111);
        assert_eq!(g.high_score(), 54_718);

        g = Game::new(30);
        g.play(5807);
        assert_eq!(g.high_score(), 37305);
    }
}
