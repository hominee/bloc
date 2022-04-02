use crate::transaction::Transaction;
use std::collections::HashMap;
use yew_agent::{Agent, AgentLink, HandlerId};

#[derive(Clone)]
pub enum Request {
    UserCreate(String),
    Transfer(Transaction),
    Transfered,
}

pub enum Response {
    UserCreate(String),
    //UserCreated,
    //UserCreateFailed,
    Transfer(Transaction),
    //Transfered,
    //TransferFailed,
}

pub struct EventBus {
    link: AgentLink<EventBus>,
    subs: HashMap<HandlerId, String>,
}

impl Agent for EventBus {
    type Reach = yew_agent::Context<Self>;
    type Message = ();
    type Input = Request;
    type Output = Request;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subs: HashMap::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        for (sub, _typ) in self.subs.iter() {
            self.link.respond(*sub, msg.clone());
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subs.insert(id, "Normal".into());
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subs.remove(&id);
    }
}
