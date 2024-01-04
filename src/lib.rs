use std::{collections::HashMap, rc::Rc, cell::RefCell};

type Module = Rc<RefCell<dyn PulseModule>>;
pub const BROADCASTER: &str = "broadcaster";

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Pulse {
    Low,
    High
}

pub struct PulseCounter {
    pub low: usize,
    pub high: usize
}

impl PulseCounter {
    pub fn new() -> Self {
        PulseCounter { low: 0, high: 0 }
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
}

pub struct SubscriptionManager {
    subscribers: Vec<Module>
}

impl SubscriptionManager {
    fn new() -> Self {
        SubscriptionManager { subscribers: Vec::new() }
    }

    fn add_subscriber(&mut self, publisher: &str, mut subscriber: Module) {
        subscriber.borrow_mut().register_input(publisher);
        self.subscribers.push(subscriber);
    }
}

pub trait PulseModule {
    fn get_id(&self) -> &str;
    fn notify(&mut self, pulse: &Pulse, sender: &str);
    fn register_input(&mut self, _: &str) {}
    fn add_subscriber(&mut self, _: Module) {}
}

pub struct FlipFlop {
    id: String,
    on: bool,
    subscription_manager: SubscriptionManager,
    pulse_counter: Rc<RefCell<PulseCounter>>
}

impl FlipFlop {
    pub fn new(id: String, pulse_counter: Rc<RefCell<PulseCounter>>) -> Self {
        FlipFlop { id, on: false, subscription_manager: SubscriptionManager::new(), pulse_counter }
    }
}

impl PulseModule for FlipFlop {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn notify(&mut self, pulse: &Pulse, _: &str) {
        self.pulse_counter.borrow_mut().increment(pulse);
        if *pulse == Pulse::Low {
            self.on = !self.on;
            let pulse_to_send = if self.on { Pulse::High } else { Pulse::Low };
            for subscriber in &mut self.subscription_manager.subscribers {
                subscriber.borrow_mut().notify(&pulse_to_send, &self.id);
            }
        }
    }

    fn add_subscriber(&mut self, subscriber: Module) {
        self.subscription_manager.add_subscriber(&self.id, subscriber);
    }
}

pub struct Conjunction {
    id: String,
    subscription_manager: SubscriptionManager,
    input_pulse_cache: HashMap<String, Pulse>,
    pulse_counter: Rc<RefCell<PulseCounter>>
}

impl Conjunction {
    pub fn new(id: String, pulse_counter: Rc<RefCell<PulseCounter>>) -> Self {
        Conjunction { id, subscription_manager: SubscriptionManager::new(), input_pulse_cache: HashMap::new(), pulse_counter }
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
        self.pulse_counter.borrow_mut().increment(pulse);
        self.input_pulse_cache.insert(sender.to_string(), *pulse);
        let pulse_to_send = if self.input_pulse_cache.values().all(|&pulse| pulse == Pulse::High) {
            Pulse::Low
        } else {
            Pulse::High
        };
        for subscriber in &mut self.subscription_manager.subscribers {
            subscriber.borrow_mut().notify(&pulse_to_send, &&self.id);
        }
    }

    fn add_subscriber(&mut self, subscriber: Module) {
        self.subscription_manager.add_subscriber(&self.id, subscriber);
    }
}

pub struct Broadcaster {
    id: String,
    subscription_manager: SubscriptionManager,
    pulse_counter: Rc<RefCell<PulseCounter>>
}

impl Broadcaster {
    pub fn new(id: String, pulse_counter: Rc<RefCell<PulseCounter>>) -> Self {
        Broadcaster { id, subscription_manager: SubscriptionManager::new(), pulse_counter }
    }
}

impl PulseModule for Broadcaster {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn notify(&mut self, pulse: &Pulse, sender: &str) {
        self.pulse_counter.borrow_mut().increment(pulse);
        for subscriber in &mut self.subscription_manager.subscribers {
            subscriber.borrow_mut().notify(pulse, sender);
        }
    }

    fn add_subscriber(&mut self, subscriber: Module) {
        self.subscription_manager.add_subscriber(&self.id, subscriber);
    }
}

pub struct Output {
    id: String,
    pulse_counter: Rc<RefCell<PulseCounter>>
}

impl Output {
    pub fn new(id: String, pulse_counter: Rc<RefCell<PulseCounter>>) -> Self {
        Output { id, pulse_counter }
    }
}

impl PulseModule for Output {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn notify(&mut self, pulse: &Pulse, _: &str) {
        self.pulse_counter.borrow_mut().increment(pulse);
    }
}
