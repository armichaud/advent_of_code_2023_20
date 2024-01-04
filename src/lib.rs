use std::{collections::HashMap, rc::Rc, cell::RefCell};

type Module = Rc<RefCell<dyn PulseModule>>;
pub const BROADCASTER: &str = "broadcaster";

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Pulse {
    Low,
    High
}

pub struct PulseManager {
    pub low: usize,
    pub high: usize,
    pub queue: Vec<(Pulse, String, Module)>
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
}

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
}
