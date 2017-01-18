use queues::publisher::Subscriber;

pub trait PubSub<T> {
    fn subscribe(&mut self) -> Subscriber<T>;
    fn add_subscriber(&mut self, s: Subscriber<T>);
}