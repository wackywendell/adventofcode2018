#![warn(clippy::all)]

struct Game {
    marbles: Vec<i64>,
    current_ix: usize,
    marble: i64,
    scores: Vec<i64>,
}

impl Game {
    fn new(players: usize) -> Game {
        Game {
            marbles: vec![0],
            current_ix: 0,
            marble: 1,
            scores: vec![0; players],
        }
    }

    fn next(&mut self) {
        if self.marble % 23 == 0 {
            self.current_ix = (self.current_ix + self.marbles.len() - 7) % self.marbles.len();
            let removed = self.marbles.remove(self.current_ix);
            let player = (self.marble as usize) % (self.scores.len());

            self.scores[player] += self.marble + removed;
            self.marble += 1;
            return;
        }

        self.current_ix = (self.current_ix + 1) % self.marbles.len() + 1;
        self.marbles.insert(self.current_ix, self.marble);
        self.marble += 1;
    }

    fn play(&mut self, nrounds: usize) {
        for _ in 0..nrounds {
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
    for i in 0..100 {
        g.play(71320);
        println!(".. {}", i);
    }
    println!("High score again: {:?}", g.high_score());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game() {
        let mut g = Game::new(9);
        g.play(25);
        assert_eq!(
            g.marbles,
            vec!(
                0, 16, 8, 17, 4, 18, 19, 2, 24, 20, 25, 10, 21, 5, 22, 11, 1, 12, 6, 13, 3, 14, 7,
                15
            )
        );
        assert_eq!(g.high_score(), 32);
    }

    #[test]
    fn test_high_scores() {
        let mut g = Game::new(10);
        g.play(1618);
        assert_eq!(g.high_score(), 8317);

        g = Game::new(13);
        g.play(7999);
        assert_eq!(g.high_score(), 146373);

        g = Game::new(17);
        g.play(1104);
        assert_eq!(g.high_score(), 2764);

        g = Game::new(21);
        g.play(6111);
        assert_eq!(g.high_score(), 54718);

        g = Game::new(30);
        g.play(5807);
        assert_eq!(g.high_score(), 37305);
    }
}
