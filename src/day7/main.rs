#![warn(clippy::all)]

#[macro_use]
extern crate lazy_static;

use clap::{App, Arg};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::iter::FromIterator;
use std::str::FromStr;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Dependency {
    parent: String,
    child: String,
}

#[derive(Copy, Clone, Debug)]
struct DependencyError;

impl FromStr for Dependency {
    type Err = DependencyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref re_shift: regex::Regex =
                regex::Regex::new(r"^Step (\w+) must be finished before step (\w+) can begin.$")
                    .unwrap();
        }

        let to_string = |i: usize, c: &regex::Captures| -> String {
            c.get(i)
                .unwrap_or_else(|| panic!("Group {} not found", i))
                .as_str()
                .parse()
                .unwrap_or_else(|_| panic!("Couldn't parse group {}", i))
        };

        match re_shift.captures(s) {
            Some(ref c) => Ok(Dependency {
                child: to_string(2, c),
                parent: to_string(1, c),
            }),
            None => Err(DependencyError),
        }
    }
}

#[derive(Debug)]
struct Graph {
    dependencies: Vec<Dependency>,
}

#[derive(Debug)]
struct DependencyMaps {
    // Maps each node to its children
    children: HashMap<String, HashSet<String>>,
    // Maps each node to its parents
    parents: HashMap<String, HashSet<String>>,
}

impl Graph {
    fn as_maps(&self) -> DependencyMaps {
        let mut children: HashMap<String, HashSet<String>> = HashMap::new();
        let mut parents: HashMap<String, HashSet<String>> = HashMap::new();
        for dep in &self.dependencies {
            // make sure the parent also at least has an empty list of parents
            parents.entry(dep.parent.clone()).or_default();
            // Insert child as child of the parent
            let c = children.entry(dep.parent.clone()).or_default();
            c.insert(dep.child.clone());

            // make sure the child also at least has an empty list of children
            children.entry(dep.child.clone()).or_default();
            // Insert parent as parent of the child
            let p = parents.entry(dep.child.clone()).or_default();
            p.insert(dep.parent.clone());
        }

        DependencyMaps {
            children: children,
            parents: parents,
        }
    }

    fn breadth_first(&self) -> Vec<String> {
        let mut deps = self.as_maps();

        let mut ready: Vec<String> = Vec::new();
        let mut finished: Vec<String> = Vec::new();
        for (n, ps) in &deps.parents {
            if ps.is_empty() {
                ready.push(n.clone());
            }
        }

        while !ready.is_empty() {
            // Keep it reverse sorted, so we can pop the earliest-by-alphabetical element
            ready.sort_by(|n1, n2| n2.cmp(n1));
            let n = ready.pop().unwrap();
            finished.push(n.clone());
            deps.parents.remove(&n);
            let children: HashSet<String> = deps.children.remove(&n).unwrap();
            for c in children {
                let ps = deps.parents.get_mut(&c).unwrap();
                ps.remove(&n);
                if ps.is_empty() {
                    ready.push(c.clone());
                }
            }
        }

        if !deps.parents.is_empty() || !deps.children.is_empty() {
            panic!(
                "Didn't empty dependency lists! Still left: {}, {}",
                deps.parents.len(),
                deps.children.len()
            )
        }

        finished
    }
}

impl<'a, S: AsRef<str>> FromIterator<S> for Graph {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let mut v = Vec::from_iter(
            iter.into_iter()
                .map(|s| Dependency::from_str(s.as_ref()).expect("ugh")),
        );
        v.sort();
        Graph { dependencies: v }
    }
}

fn main() -> std::io::Result<()> {
    let matches = App::new("Day 7")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day7.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);

    let graph = Graph::from_iter(buf_reader.lines().filter_map(|l| l.ok()));

    let finished = graph.breadth_first();

    println!("Order: {}", finished.join(""));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() {
        let lines = vec![
            "Step C must be finished before step A can begin.",
            "Step C must be finished before step F can begin.",
            "Step A must be finished before step B can begin.",
            "Step A must be finished before step D can begin.",
            "Step B must be finished before step E can begin.",
            "Step D must be finished before step E can begin.",
            "Step F must be finished before step E can begin.",
        ];

        let graph = Graph::from_iter(lines);
        let finished = graph.breadth_first();
        assert_eq!("CABDFE", finished.join(""));
    }
}
