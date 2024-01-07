use std::{collections::{HashMap, HashSet}, fs::File, io::{BufReader, BufRead}};

const BROADCASTER: &str = "broadcaster";

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum PulseLevel {
    Low,
    High
}

#[derive(Debug, Clone)]
struct Pulse {
    level: PulseLevel,
    source: String
}

#[derive(Debug, Clone)]
struct PulseManager {
    low_count: usize,
    high_count: usize,
    queue: Vec<Pulse>,
    subscriptions: HashMap<String, Vec<String>>
}

impl PulseManager {
    fn new() -> PulseManager {
        PulseManager {
            low_count: 0,
            high_count: 0,
            queue: Vec::new(),
            subscriptions: HashMap::new()
        }
    }

    fn get_suscribers(&self, source: &str) -> Vec<String> {
        self.subscriptions.get(source).unwrap().clone()
    }

    fn run(&mut self, modules: &mut HashMap<String, PulseModule>, n: usize) {
        for _ in 0..n {
            self.run_once(modules);
        }
    }

    fn run_once(&mut self, modules: &mut HashMap<String, PulseModule>) {
        self.send(Pulse {
            level: PulseLevel::Low,
            source: BROADCASTER.to_string()
        });
        while self.queue.len() > 0 {
            let pulse = self.queue.remove(0);
            let subscribers = self.subscriptions.get(&pulse.source).unwrap().clone();
            for subscriber in subscribers {
                self.increment(pulse.level);
                let module = modules.get_mut(&subscriber).unwrap();
                if let Some(next_pulse) = module.receive(pulse.clone()) {
                    self.send(next_pulse);
                }
            }
        }
    }

    fn send(&mut self, pulse: Pulse) {
        self.queue.push(pulse);
    }

    fn add_subscribers(&mut self, source: &str, targets: &[&str]) {
        self.subscriptions.insert(source.to_string(), targets.iter().map(|&x| x.to_string()).collect());
    }

    fn increment(&mut self, level: PulseLevel) {
        match level {
            PulseLevel::Low => self.low_count += 1,
            PulseLevel::High => self.high_count += 1
        }
    }

    fn pulse_products(&self) -> usize {
        self.low_count * self.high_count
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum PulseModuleType {
    FlipFlop,
    Conjunction,
    Broadcaster,
    Output
}

impl PulseModuleType {
    fn from_raw_id(id: &str) -> PulseModuleType {
        if id == BROADCASTER {
            PulseModuleType::Broadcaster 
        } else if id.starts_with("&") {
            PulseModuleType::Conjunction
        } else if id.starts_with("%") {
            PulseModuleType::FlipFlop
        } else {
            PulseModuleType::Output
        }
    }
}

#[derive(Debug, Clone)]
struct PulseModule {
    module_type: PulseModuleType,
    id: String,
    input_memory: HashMap<String, PulseLevel>,
    on: bool
}

impl PulseModule {
    fn new(module_type: PulseModuleType, id: String) -> PulseModule {
        PulseModule {
            module_type,
            id,
            input_memory: HashMap::new(),
            on: false
        }
    }

    fn conjunction_output(&self) -> PulseLevel {
        return if self.input_memory.values().all(|&x| x == PulseLevel::High) {
            PulseLevel::High
        } else {
            PulseLevel::Low
        }
    }

    fn receive(&mut self, pulse: Pulse) -> Option<Pulse> {
        self.input_memory.insert(pulse.source, pulse.level);
        let level = match self.module_type {
            PulseModuleType::FlipFlop => { 
                if pulse.level == PulseLevel::High {
                    return None;
                }
                if self.on {
                    self.on = false;
                    Some(PulseLevel::Low)
                } else {
                    self.on = true;
                    Some(PulseLevel::High)
                }
            },
            PulseModuleType::Conjunction => Some(self.conjunction_output()),
            PulseModuleType::Broadcaster => Some(pulse.level),
            PulseModuleType::Output => None
        };
        if let Some(level) = level {
            return Some(Pulse { 
                level,
                source: self.id.clone()
            });
        }
        None
    }
} 

fn solution(file: &str) -> usize {
    let mut manager = PulseManager::new();
    let mut modules: HashMap<String, PulseModule> = HashMap::new();

    // Parse input
    let file = File::open(file).unwrap();
    let lines = BufReader::new(file).lines();
    let mut complete_module_list = HashSet::<String>::new();
    for line in lines {
        let line = line.unwrap();
        let mut split = line.split(" -> ");
        let publisher = split.next().unwrap();
        let publisher_type = PulseModuleType::from_raw_id(publisher);
        let mut id = publisher.to_string();
        if vec![PulseModuleType::Conjunction, PulseModuleType::FlipFlop].contains(&publisher_type) {
            id = id[1..].to_string();
        }
        let module = PulseModule::new(publisher_type, id.clone());
        modules.insert(id.clone(), module);
        let targets = split.next().unwrap().split(", ").collect::<Vec<&str>>();
        manager.add_subscribers(id.as_str(), &targets);
        complete_module_list.extend(&mut targets.iter().map(|&x| x.to_string()));
    }
    let output_modules = complete_module_list
        .iter()
        .filter(|&x| !modules.keys().map(|x| x.to_owned()).collect::<Vec<String>>().contains(x))
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();
    for id in output_modules {
        modules.insert(id.clone(), PulseModule::new(PulseModuleType::Output, id));
    }
    let complete_module_list = modules.keys()
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();
    for id in complete_module_list {
        manager.get_suscribers(id.as_str()).iter().for_each(|subscriber| {
            modules.get_mut(subscriber).unwrap().input_memory.insert(id.clone(), PulseLevel::Low);
        })   
    }
    manager.run(&mut modules, 1000);
    manager.pulse_products()
}

fn main() {
    assert_eq!(solution("example_1.txt"), 32000000);
    // assert_eq!(solution("example_2.txt"), 11687500);
    //assert_eq!(solution("input.txt"), 0);
}
