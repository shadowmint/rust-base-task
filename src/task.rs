use ::{TaskFunc, TaskError, TaskErrorCode};
use ::futures::sync::oneshot::{channel, Receiver};
use std::panic::{catch_unwind, UnwindSafe};

/// Task is a logical unit of work that can be passed through channels and to other threads
/// to be executed. Resolve into a Result<T, TErr> after they have been invoked.
pub struct Task {
    action: Box<TaskFunc>,
}

impl Task {
    /// Create a new task from a unit of work
    pub fn new<T, TFn>(action: TFn) -> (Task, Receiver<Result<T, TaskError>>) where TFn: FnOnce() -> Result<T, TaskError> + Send + UnwindSafe + 'static, T: Send + UnwindSafe + 'static {
        let (sx, rx) = channel::<Result<T, TaskError>>();
        let task = Task {
            action: Box::new(move || {
                let safe_result = catch_unwind(move || {
                    return action();
                });
                match safe_result {
                    Ok(result) => {
                        match sx.send(result) {
                            Ok(_) => {}
                            Err(_) => {}
                        };
                    }
                    Err(_) => {
                        match sx.send(Err(TaskError::from(TaskErrorCode::Panic))) {
                            Ok(_) => {}
                            Err(_) => {}
                        };
                    }
                }
            })
        };
        return (task, rx);
    }

    /// Invoke this task, consuming the task and dispatching the result.
    pub fn call_once(self) {
        self.action.call_once()
    }
}

#[cfg(test)]
mod tests {
    use super::Task;
    use ::futures::Future;
    use ::futures::Async;
    use std::thread;
    use std::time::Duration;
    use ::TaskError;

    #[test]
    fn test_resolve_local_task() {
        let (foo, mut foo_recv) = Task::new(|| {
            return Ok(100);
        });

        foo.call_once();
        let result = foo_recv.poll().ok().unwrap();
        match result {
            Async::Ready(t) => {
                assert_eq!(t.ok().unwrap(), 100);
            }
            Async::NotReady => { unreachable!() }
        };
    }

    #[test]
    fn test_resolve_task_across_threads() {
        let (foo, mut foo_recv) = Task::new(|| {
            return Ok(100);
        });

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            foo.call_once();
        });

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            let result = foo_recv.poll().ok().unwrap();
            match result {
                Async::Ready(t) => {
                    assert_eq!(t.ok().unwrap(), 100);
                }
                Async::NotReady => { unreachable!() }
            };
        });
    }

    #[test]
    fn test_resolve_local_task_with_error() {
        let (foo, mut foo_recv) = Task::new(|| {
            if false {
                return Ok(0);
            }
            return Err(TaskError::from("Aborted"));
        });

        foo.call_once();
        let result = foo_recv.poll().ok().unwrap();
        match result {
            Async::Ready(t) => {
                assert!(t.is_err());
            }
            Async::NotReady => { unreachable!() }
        };
    }

    #[test]
    fn test_resolve_local_task_with_panic() {
        let (foo, mut foo_recv) = Task::new(|| {
            if false {
                return Ok(0);
            }
            //panic::set_hook(Box::new(|_| {}));
            panic!();
        });

        foo.call_once();
        let result = foo_recv.poll().ok().unwrap();
        match result {
            Async::Ready(t) => {
                assert!(t.is_err());
            }
            Async::NotReady => { unreachable!() }
        };
    }
}
