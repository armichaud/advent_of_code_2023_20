use std::{collections::HashMap, rc::Rc, cell::RefCell};

pub const BROADCASTER: &str = "broadcaster";

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Pulse {
    Low,
    High
}

pub struct PulseCounter {
    Low: usize,
    High: usize
}

impl PulseCounter {
    pub fn new() -> Self {
        PulseCounter { Low: 0, High: 0 }
    }

    fn increment(&mut self, pulse: &Pulse) {
        match pulse {
            Pulse::Low => self.Low += 1,
            Pulse::High => self.High += 1
        }
    }
}

pub trait Subscribe {
    fn add_subscriber(&mut self, publisher: &str, subscriber: Box<dyn PulseModule>);
}

pub struct SubscriptionManager {
    subscribers: Vec<Box<dyn PulseModule>>
}

impl SubscriptionManager {
    fn new() -> Self {
        SubscriptionManager { subscribers: Vec::new() }
    }
}

impl Subscribe for SubscriptionManager {
    fn add_subscriber(&mut self, publisher: &str, mut subscriber: Box<dyn PulseModule>) {
        subscriber.register_input(publisher);
        self.subscribers.push(subscriber);
    }
}

pub trait PulseModule {
    fn notify(&mut self, pulse: &Pulse, sender: &str);
    fn register_input(&mut self, input: &str) {}
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
    fn notify(&mut self, pulse: &Pulse, _: &str) {
        self.pulse_counter.borrow_mut().increment(pulse);
        if *pulse == Pulse::Low {
            self.on = !self.on;
            let pulse_to_send = if self.on { Pulse::High } else { Pulse::Low };
            for subscriber in &mut self.subscription_manager.subscribers {
                subscriber.notify(&pulse_to_send, &self.id);
            }
        }
    }
}

impl Subscribe for FlipFlop {
    fn add_subscriber(&mut self, _: &str, subscriber: Box<dyn PulseModule>) {
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

impl Subscribe for Conjunction {
    fn add_subscriber(&mut self, _: &str, subscriber: Box<dyn PulseModule>) {
        self.subscription_manager.add_subscriber(&self.id, subscriber);
    }
}

impl PulseModule for Conjunction {
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
            subscriber.notify(&pulse_to_send, &&self.id);
        }
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
    fn notify(&mut self, pulse: &Pulse, sender: &str) {
        self.pulse_counter.borrow_mut().increment(pulse);
        for subscriber in &mut self.subscription_manager.subscribers {
            subscriber.notify(pulse, sender);
        }
    }
}

impl Subscribe for Broadcaster {
    fn add_subscriber(&mut self, _: &str, subscriber: Box<dyn PulseModule>) {
        self.subscription_manager.add_subscriber(&self.id, subscriber);
    }
}
