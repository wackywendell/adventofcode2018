#![warn(clippy::all)]

#[macro_use]
extern crate itertools;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
struct Grid(i64);

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
struct MaxPower {
    x: i64,
    y: i64,
    size: i64,
    power: i64,
}

impl Grid {
    fn power(self, x: i64, y: i64) -> i64 {
        let rack_id = x + 10;
        let power1 = ((rack_id * y) + self.0) * rack_id;
        let hundreds_digit = (power1 % 1000) / 100;
        hundreds_digit - 5
    }

    fn cell_power(self, x: i64, y: i64, w: i64, h: i64) -> i64 {
        iproduct!(0..w, 0..h)
            .map(|(dx, dy)| self.power(x + dx, y + dy))
            .sum()
    }

    /*

    (2, 2)    ---->   (2, 3)
    - - - -           - - - -
    - # # #           - - - -
    - # # #   ---->   - # # #
    - # # #           - # # #
    - - - -           - # # #
    - - - -           - - - -

    Need to add (2, 5) through (4, 5) - (2, 2) through (4,2)
    (x, y+size-1) through (x+size-1, y+size-1) - (x, y-1) through (x+size-1, y-1)
    */

    fn max_power(self, size: i64) -> MaxPower {
        let mut max = MaxPower {
            x: 1,
            y: 1,
            size,
            power: self.cell_power(1, 1, size, size),
        };
        for x in 1..=300 - size {
            let mut power = self.cell_power(x, 1, size, size);
            for y in 1..=300 - size {
                if y > 1 {
                    for nx in x..x + size {
                        power += self.power(nx, y + size - 1) - self.power(nx, y - 1)
                    }
                }
                if power > max.power {
                    max = MaxPower { x, y, size, power }
                }
            }
        }

        max
    }

    fn max_up_to_power(self, max_size: i64) -> MaxPower {
        let mut max = None;
        for size in 1..=max_size {
            let current = self.max_power(size);
            max = match max {
                Some(MaxPower { power, .. }) if current.power < power => max,
                _ => Some(current),
            }
        }

        max.expect("There should be at least one")
    }

    fn max_any_power(self) -> MaxPower {
        self.max_up_to_power(300)
    }
}

fn main() -> std::io::Result<()> {
    let g = Grid(3463);

    let MaxPower { x, y, power, .. } = g.max_power(3);
    println!("Found power {} at ({}, {})", power, x, y);

    let MaxPower { x, y, power, size } = g.max_any_power();
    println!("Found power {} for identifier {},{},{}", power, x, y, size);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_grid_power() {
        assert_eq!(Grid(8).power(3, 5), 4);
        assert_eq!(Grid(57).power(122, 79), -5);
        assert_eq!(Grid(39).power(217, 196), 0);
        assert_eq!(Grid(71).power(101, 153), 4);
    }

    #[test]
    fn test_max_power() {
        assert_eq!(
            Grid(18).max_power(3),
            MaxPower {
                x: 33,
                y: 45,
                power: 29,
                size: 3
            }
        );
        assert_eq!(
            Grid(42).max_power(3),
            MaxPower {
                x: 21,
                y: 61,
                power: 30,
                size: 3,
            }
        );
    }

    #[test]
    fn test_max_any_power() {
        assert_eq!(
            Grid(18).max_up_to_power(16),
            MaxPower {
                x: 90,
                y: 269,
                power: 113,
                size: 16,
            }
        );
        assert_eq!(
            Grid(42).max_up_to_power(12),
            MaxPower {
                x: 232,
                y: 251,
                power: 119,
                size: 12,
            }
        );
    }
}
