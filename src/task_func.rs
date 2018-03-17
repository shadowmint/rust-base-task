/// TaskFunc allows a boxed instance to consume itself by being invoked.
pub trait TaskFunc: Send {
    fn call_once(self: Box<Self>);
}

// Implement TaskFunc for all FnOnce.
impl<TFn: FnOnce() -> () + Send> TaskFunc for TFn {
    fn call_once(self: Box<Self>) {
        (*self)()
    }
}