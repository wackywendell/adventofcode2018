#![warn(clippy::all)]

use clap::App;

type Recipe = i16;

struct Recipes {
    first: usize,
    second: usize,
    recipes: Vec<Recipe>,
}

impl Recipes {
    fn new(first: Recipe, second: Recipe) -> Self {
        Recipes {
            first: 0,
            second: 1,
            recipes: vec![first, second],
        }
    }

    fn step(&mut self) -> (Option<Recipe>, Recipe) {
        let (r1, r2) = (self.recipes[self.first], self.recipes[self.second]);
        let sum = r1 + r2;
        let (n1, n2) = if sum < 10 {
            (None, sum)
        } else {
            (Some(sum / 10), sum % 10)
        };

        if let Some(r) = n1 {
            self.recipes.push(r);
        }
        self.recipes.push(n2);

        self.first = (self.first + r1 as usize + 1) % self.recipes.len();
        self.second = (self.second + r2 as usize + 1) % self.recipes.len();

        (n1, n2)
    }

    fn get_string(&mut self, start_ix: usize, len: usize) -> String {
        self.advance_to(start_ix + len);

        let mut s = String::with_capacity(10);
        for r in self.recipes.iter().skip(start_ix).take(len) {
            s.push_str(&r.to_string());
        }

        s
    }

    fn advance_to(&mut self, len: usize) {
        while self.recipes.len() < len {
            self.step();
        }
    }

    fn check_match(&self, needle: &[Recipe], ix: usize) -> bool {
        for (r1, r2) in needle.iter().zip(self.recipes.iter().skip(ix)) {
            if r1 != r2 {
                return false;
            }
        }

        true
    }

    fn find_set(&mut self, needle: &[Recipe]) -> usize {
        let n_len = needle.len();
        while self.recipes.len() < n_len {
            self.step();
        }

        for ix in 0..=(self.recipes.len() - n_len) {
            if self.check_match(needle, ix) {
                return ix;
            }
        }

        loop {
            let (r1, _) = self.step();
            let ix = self.recipes.len() - needle.len();
            if r1.is_some() && self.check_match(needle, ix - 1) {
                return ix - 1;
            }
            if self.check_match(needle, ix) {
                return ix;
            }
        }
    }
}

impl std::fmt::Display for Recipes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (n, r) in self.recipes.iter().enumerate() {
            match n {
                n if n == self.first => write!(f, "({})", r),
                n if n == self.second => write!(f, "[{}]", r),
                _ => write!(f, " {} ", r),
            }?
        }

        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let _matches = App::new("Day 14").get_matches();

    let input = 939_601;
    let split: Vec<Recipe> = input
        .to_string()
        .chars()
        .map(|c| (c.to_digit(10).unwrap() as Recipe))
        .collect();

    eprintln!("Using input {}", input);

    let mut rs = Recipes::new(3, 7);
    // println!("Recipes:\n{}", rs);
    // for _ in 0..10 {
    //     rs.step();
    //     println!("{}", rs);
    // }
    println!(
        "Last ten recipes after {}: {}",
        input,
        rs.get_string(input, 10)
    );

    let found = rs.find_set(split.as_slice());
    println!("Found {} after: {}", input, found);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipes() {
        let mut r = Recipes::new(3, 7);

        r.advance_to(20);
        println!("{}", r);
        assert_eq!(r.get_string(5, 10), "0124515891");
        assert_eq!(r.get_string(9, 10), "5158916779");
        assert_eq!(r.get_string(18, 10), "9251071085");
        assert_eq!(r.get_string(2018, 10), "5941429882");
    }

    #[test]
    fn test_find() {
        let mut r = Recipes::new(3, 7);
        r.advance_to(20);

        assert_eq!(r.find_set(&[0, 1, 2, 4, 5]), 5);
        assert_eq!(r.find_set(&[5, 1, 5, 8, 9]), 9);
        assert_eq!(r.find_set(&[9, 2, 5, 1, 0]), 18);
        assert_eq!(r.find_set(&[5, 9, 4, 1, 4]), 2018);
    }
}
