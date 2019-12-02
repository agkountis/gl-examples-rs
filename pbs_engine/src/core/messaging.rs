use std::any::Any;

/// A trait that allows any struct to listen to specific types of messages.
pub trait Subscriber<T> {
    fn on_message(&mut self, message: &T);
}

/// A topic represents a category of messages.
/// Each topic maintains a vector of subscribers (in the form of closures)
/// that notify and propagate the messages published to this topic to all subscribers.
pub struct Topic {
    id: i32,
    subscribers: Vec<Box<dyn FnMut(&dyn Any)>>,
}

impl Topic {
    pub fn new(id: i32) -> Self {
        Topic {
            id,
            subscribers: vec![]
        }
    }
}

/// A struct that maintains a list of Topics where messages can be published
pub struct Broker {
    topics: Vec<Topic>
}

impl Broker {
    /// Constructs a Broker with no Topics.
    /// add_topic(...) can be used to add new Topics to the Broker after construction.
    pub fn new() -> Self {
        Broker {
            topics: vec![]
        }
    }

    /// Constructs a Broker with the specified Topics.
    pub fn new_with_topics(topics: Vec<Topic>) -> Self {
        Broker {
            topics
        }
    }

    /// Adds a topic to the broker.
    pub fn add_topic(&mut self, topic: Topic) {
        self.topics.push(topic)
    }

    /// Publishes a message to the specified subject.
    /// The message can be of any type.
    pub fn publish<T>(&mut self, topic_id: i32, message: T)
        where T: 'static {

        if let Some(ref mut topic) = self.topics.iter_mut().find(|t| {
            t.id == topic_id
        }) {
            topic.subscribers.iter_mut().for_each(|notify|{
                notify(&message)
            })
        }

    }

    /// Adds a subscription to the broker for the specified Topic.
    /// If the Topic does not exist it is created.
    pub fn subscribe<F>(&mut self, topic_id: i32, closure: F)
        where F: FnMut(&dyn Any) + 'static {

        if let Some(topic) = self.topics.iter_mut().find(|topic| {
            topic_id == topic.id
        }) {
            topic.subscribers.push(Box::new(closure));
        }
        else {
            println!("Topic with id: {} does not exist! Creating...", topic_id);

            let mut topic = Topic::new(topic_id);
            topic.subscribers.push(Box::new(closure));
            self.topics.push(topic)
        }

    }
}
