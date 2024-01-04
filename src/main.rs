use std::{fs::File, io::{BufReader, BufRead}, collections::HashMap, cell::RefCell, rc::Rc};

use crate::lib::*;

mod lib;

fn solution(file: &str) -> usize {
    let file = File::open(file).unwrap();
    let lines = BufReader::new(file).lines();
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for line in lines {
        let line = line.unwrap();
        let mut split = line.split("->");
        let id = split.next().unwrap().trim().to_owned();
        let inputs = split.next().unwrap().trim().split(",").map(|chars| chars.trim().to_owned()).collect::<Vec<String>>();
        map.insert(id, inputs);
    }
    let pulse_counter = Rc::new(RefCell::new(PulseCounter::new()));
    let mut broadcaster: Broadcaster;
    let mut module_map: HashMap<String, Box<dyn PulseModule>> = HashMap::new();
    for id in map.keys() {
        if id == BROADCASTER {
            broadcaster = Broadcaster::new(id.to_owned(), Rc::clone(&pulse_counter));
        } else if id.starts_with("&") {
            module_map.insert(id[1..].to_owned(), Box::new(Conjunction::new(id.to_owned(), Rc::clone(&pulse_counter))));
        } else {
            module_map.insert(id[1..].to_owned(), Box::new(FlipFlop::new(id.to_owned(), Rc::clone(&pulse_counter))));
        }
    }
    // TODO
    // Set up subscriptions
    0
}

fn main() {
    assert_eq!(solution("example_1.txt"), 32000000);
    assert_eq!(solution("example_2.txt"), 11687500);
    assert_eq!(solution("input.txt"), 0);
}
