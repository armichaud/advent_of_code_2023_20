use std::{fs::File, io::{BufReader, BufRead}, collections::HashMap, cell::RefCell, rc::Rc, borrow::Borrow};
use advent_of_code_2023_20::*;

fn solution(file: &str) -> usize {
    let file = File::open(file).unwrap();
    let lines = BufReader::new(file).lines();
    let mut publisher_map: HashMap<String, Vec<String>> = HashMap::new();
    for line in lines {
        let line = line.unwrap();
        let mut split = line.split("->");
        let id = split.next().unwrap().trim().to_owned();
        let inputs = split.next().unwrap().trim().split(",").map(|chars| chars.trim().to_owned()).collect::<Vec<String>>();
        publisher_map.insert(id, inputs);
    }
    let pulse_counter: Rc<RefCell<PulseCounter>> = Rc::new(RefCell::new(PulseCounter::new()));
    let mut broadcaster: Option<Rc<RefCell<dyn PulseModule>>> = None;
    let mut module_map: HashMap<String, Rc<RefCell<dyn PulseModule>>> = HashMap::new();
    for id in publisher_map.keys() {
        if id == BROADCASTER {
            let broadcaster_rc: Rc<RefCell<dyn PulseModule>> = Rc::new(RefCell::new(Broadcaster::new(id.to_owned(), Rc::clone(&pulse_counter))));
            broadcaster = Some(Rc::clone(&broadcaster_rc));
            module_map.insert(id.to_owned(), Rc::clone(&broadcaster_rc));
        } else if id.starts_with("&") {
            module_map.insert(id[1..].to_owned(), Rc::new(RefCell::new(Conjunction::new(id.to_owned(), Rc::clone(&pulse_counter)))));
        } else {
            module_map.insert(id[1..].to_owned(), Rc::new(RefCell::new(FlipFlop::new(id.to_owned(), Rc::clone(&pulse_counter)))));
        }
    }
    if broadcaster.is_none() {
        panic!("No broadcaster found");
    }
    let broadcaster = broadcaster.unwrap();
    for subscribers in publisher_map.values() {
        for subscriber_id in subscribers {
            if module_map.get(subscriber_id).is_none() {
                module_map.insert(subscriber_id.to_string(), Rc::new(RefCell::new(Output::new(subscriber_id.to_owned(), Rc::clone(&pulse_counter)))));
            }
        }
    }
    for publisher in module_map.values() {
        let mut publisher = publisher.borrow_mut();
        for subscriber_id in publisher_map.get(publisher.get_id()).unwrap().iter() {
            let subscriber = module_map.get(subscriber_id).unwrap();
            publisher.add_subscriber(Rc::clone(&subscriber));
        }
    }
    for _ in 0..100 {
        broadcaster.borrow_mut().notify(&Pulse::Low, BROADCASTER);
    }
    let final_counts: &RefCell<PulseCounter> = pulse_counter.borrow();
    let final_counts = final_counts.borrow();
    final_counts.get_product_of_counts()
}

fn main() {
    assert_eq!(solution("example_1.txt"), 32000000);
    assert_eq!(solution("example_2.txt"), 11687500);
    assert_eq!(solution("input.txt"), 0);
}
