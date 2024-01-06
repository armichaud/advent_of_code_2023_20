use std::{collections::HashMap, rc::Rc, cell::RefCell};

type Module = Rc<RefCell<dyn PulseModule>>;
pub const BROADCASTER: &str = "broadcaster";

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Pulse {
    Low,
    High
}

pub struct PulseManager {
    pub low: usize,
    pub high: usize,
    pub queue: Vec<(Pulse, String, Module)>
}

impl std::fmt::Debug for PulseManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PulseManager").finish()
    }
}

impl PulseManager {
    pub fn new() -> Self {
        PulseManager { low: 0, high: 0, queue: Vec::new() }
    }

    fn increment(&mut self, pulse: &Pulse) {
        match pulse {
            Pulse::Low => self.low += 1,
            Pulse::High => self.high += 1
        }
    }

    pub fn get_product_of_counts(&self) -> usize {
        self.low * self.high
    }

    fn notify_all(&mut self, pulse: &Pulse, sender: &str, subscribers: &Vec<Module>) {
        for subscriber in subscribers {
            self.queue.push((*pulse, sender.to_owned(), Rc::clone(&subscriber)));
        }
    }

    pub fn run(&mut self, broadcaster: Module) {
        self.queue.push((Pulse::Low, BROADCASTER.to_owned(), broadcaster));
        while self.queue.len() > 0 {
            let (pulse, sender, module) = self.queue.remove(0);
            module.borrow_mut().notify(&pulse, &sender);
        }
    }
}


pub struct SubscriptionManager {
    subscribers: Vec<Module>
}

impl std::fmt::Debug for SubscriptionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubscriptionManager")
            .field("subscriber count", &self.subscribers.len())
            .field("subscriber ids", &self.subscribers.iter().map(|subscriber| subscriber.borrow().get_id().to_owned()).collect::<Vec<String>>())
            .finish()
    }
}

impl SubscriptionManager {
    fn new() -> Self {
        SubscriptionManager { subscribers: Vec::new() }
    }

    fn add_subscriber(&mut self, publisher: &str, subscriber: Module) {
        subscriber.borrow_mut().register_input(publisher);
        self.subscribers.push(subscriber);
    }

    fn get_all_subsribers(&self) -> Vec<Module> {
        self.subscribers.clone()
    }
}

pub trait PulseModule {
    fn get_id(&self) -> &str;
    fn notify(&mut self, pulse: &Pulse, sender: &str);
    fn register_input(&mut self, _: &str) {}
    fn add_subscriber(&mut self, subscriber: Module);
    fn update_queue(&mut self, pulse: &Pulse);
    fn get_subscription_manager(&self) -> Option<&SubscriptionManager>;
}

impl std::fmt::Debug for dyn PulseModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PulseModule")
            .field("id", &self.get_id())
            .field("subscriptions", &self.get_subscription_manager().unwrap_or(&SubscriptionManager::new()))
            .finish()
    }
}

#[derive(Debug)]
pub struct FlipFlop {
    id: String,
    on: bool,
    subscription_manager: SubscriptionManager,
    pulse_manager: Rc<RefCell<PulseManager>>
}

impl FlipFlop {
    pub fn new(id: String, pulse_manager: Rc<RefCell<PulseManager>>) -> Self {
        FlipFlop { id, on: false, subscription_manager: SubscriptionManager::new(), pulse_manager }
    }
}

impl PulseModule for FlipFlop {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_subscription_manager(&self) -> Option<&SubscriptionManager> {
        Some(&self.subscription_manager)
    }

    fn notify(&mut self, pulse: &Pulse, _: &str) {
        self.pulse_manager.borrow_mut().increment(pulse);
        if *pulse == Pulse::Low {
            self.on = !self.on;
            self.pulse_manager
                .borrow_mut()
                .notify_all(
                    if self.on { &Pulse::High } else { &Pulse::Low }, 
                    &self.id, 
                    &self.subscription_manager.get_all_subsribers()
                );
        }
    }

    fn add_subscriber(&mut self, subscriber: Module) {
        self.subscription_manager.add_subscriber(&self.id, subscriber);
    }

    fn update_queue(&mut self, pulse: &Pulse) {
        self.pulse_manager.borrow_mut().notify_all(
            &pulse, 
            &self.id, 
            &self.subscription_manager.get_all_subsribers()
        );
    }
}

pub struct Conjunction {
    id: String,
    subscription_manager: SubscriptionManager,
    input_pulse_cache: HashMap<String, Pulse>,
    pulse_manager: Rc<RefCell<PulseManager>>
}

impl Conjunction {
    pub fn new(id: String, pulse_manager: Rc<RefCell<PulseManager>>) -> Self {
        Conjunction { id, subscription_manager: SubscriptionManager::new(), input_pulse_cache: HashMap::new(), pulse_manager }
    }
}

impl PulseModule for Conjunction {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_subscription_manager(&self) -> Option<&SubscriptionManager> {
        Some(&self.subscription_manager)
    }

    fn register_input(&mut self, input: &str) {
        self.input_pulse_cache.insert(input.to_string(), Pulse::Low);
    }

    fn notify(&mut self, pulse: &Pulse, sender: &str) {
        self.pulse_manager.borrow_mut().increment(pulse);
        self.input_pulse_cache.insert(sender.to_string(), *pulse);
        let pulse_to_send = if self.input_pulse_cache.values().all(|&pulse| pulse == Pulse::High) {
            Pulse::Low
        } else {
            Pulse::High
        };
        self.update_queue(&pulse_to_send);
    }

    fn add_subscriber(&mut self, subscriber: Module) {
        self.subscription_manager.add_subscriber(&self.id, subscriber);
    }

    fn update_queue(&mut self, pulse: &Pulse) {
        self.pulse_manager.borrow_mut().notify_all(
            &pulse, 
            &self.id, 
            &self.subscription_manager.get_all_subsribers()
        );
    }
}

pub struct Broadcaster {
    id: String,
    subscription_manager: SubscriptionManager,
    pulse_manager: Rc<RefCell<PulseManager>>
}

impl Broadcaster {
    pub fn new(id: String, pulse_manager: Rc<RefCell<PulseManager>>) -> Self {
        Broadcaster { id, subscription_manager: SubscriptionManager::new(), pulse_manager }
    }
}

impl PulseModule for Broadcaster {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_subscription_manager(&self) -> Option<&SubscriptionManager> {
        Some(&self.subscription_manager)
    }

    fn notify(&mut self, pulse: &Pulse, _: &str) {
        self.pulse_manager.borrow_mut().increment(pulse);
        self.update_queue(pulse);
    }

    fn add_subscriber(&mut self, subscriber: Module) {
        self.subscription_manager.add_subscriber(&self.id, subscriber);
    }

    fn update_queue(&mut self, pulse: &Pulse) {
        self.pulse_manager.borrow_mut().notify_all(
            &pulse, 
            &self.id, 
            &self.subscription_manager.get_all_subsribers()
        );
    }
}

pub struct Output {
    id: String,
    pulse_manager: Rc<RefCell<PulseManager>>
}

impl Output {
    pub fn new(id: String, pulse_manager: Rc<RefCell<PulseManager>>) -> Self {
        Output { id, pulse_manager }
    }
}

impl PulseModule for Output {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn notify(&mut self, pulse: &Pulse, _: &str) {
        self.pulse_manager.borrow_mut().increment(pulse);
    }

    fn add_subscriber(&mut self, _: Module) {}
    fn update_queue(&mut self, _: &Pulse) {}
    fn get_subscription_manager(&self) -> Option<&SubscriptionManager> { None }
}

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
    let pulse_manager: Rc<RefCell<PulseManager>> = Rc::new(RefCell::new(PulseManager::new()));
    let mut broadcaster: Option<Rc<RefCell<dyn PulseModule>>> = None;
    let mut module_map: HashMap<String, Rc<RefCell<dyn PulseModule>>> = HashMap::new();
    for id in publisher_map.keys() {
        if id == BROADCASTER {
            let broadcaster_rc: Rc<RefCell<dyn PulseModule>> = Rc::new(RefCell::new(Broadcaster::new(id.to_owned(), Rc::clone(&pulse_manager))));
            broadcaster = Some(Rc::clone(&broadcaster_rc));
            module_map.insert(id.to_owned(), Rc::clone(&broadcaster_rc));
        } else if id.starts_with("&") {
            module_map.insert(id[1..].to_owned(), Rc::new(RefCell::new(Conjunction::new(id.to_owned(), Rc::clone(&pulse_manager)))));
        } else {
            module_map.insert(id[1..].to_owned(), Rc::new(RefCell::new(FlipFlop::new(id.to_owned(), Rc::clone(&pulse_manager)))));
        }
    }
    if broadcaster.is_none() {
        panic!("No broadcaster found");
    }
    let broadcaster = broadcaster.unwrap();
    for subscribers in publisher_map.values() {
        for subscriber_id in subscribers {
            if module_map.get(subscriber_id).is_none() {
                module_map.insert(subscriber_id.to_string(), Rc::new(RefCell::new(Output::new(subscriber_id.to_owned(), Rc::clone(&pulse_manager)))));
            }
        }
    }
    for publisher in module_map.values() {
        let mut publisher = publisher.borrow_mut();
        if let Some(subscribers) = publisher_map.get(publisher.get_id()) {
            for subscriber_id in subscribers {
                let subscriber = module_map.get(subscriber_id).unwrap();
                publisher.add_subscriber(Rc::clone(&subscriber));
            }
        }
    }
    for _ in 0..100 {
        pulse_manager.borrow_mut().run(Rc::clone(&broadcaster));
    }
    //println!("{:?}", module_map);
    let final_counts: &RefCell<PulseManager> = pulse_manager.borrow();
    let final_counts = final_counts.borrow();
    final_counts.get_product_of_counts()
}

fn main() {
    assert_eq!(solution("example_2.txt"), 32000000);
    assert_eq!(solution("example_2.txt"), 11687500);
    assert_eq!(solution("input.txt"), 0);
}
