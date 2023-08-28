trait EventListener<T> {
    fn on(&mut self, event: &str, callback: T);
}
