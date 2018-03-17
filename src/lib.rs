extern crate futures;

mod task_error;
mod task_func;
mod task;

pub use task_error::TaskError;
pub use task_error::TaskErrorCode;
pub use task_func::TaskFunc;
pub use task::Task;

#[cfg(test)]
mod tests {
    extern crate tokio;

    use std::thread;
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use ::{Task};
    use ::futures::{Future, Canceled};

    #[test]
    fn test_wait_for_task() {
        let (task, promise) = Task::new(|| {
            return Ok(1000);
        });

        let g1 = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            task.call_once();
        });

        let g2 = thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            tokio::run(promise.then(|promise_result| {
                match promise_result {
                    Ok(task_result) => {
                        match task_result {
                            Ok(value) => {
                                assert_eq!(value, 1000);
                            }
                            Err(_) => { unreachable!(); }
                        };
                    }
                    Err(_) => { unreachable!(); }
                };
                Ok(())
            }));
        });

        g2.join().unwrap();
        g1.join().unwrap();
    }

    #[test]
    fn test_send_task_over_channel() {
        let (sx, rx) = channel::<Task>();

        let g1 = thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(task) => {
                        task.call_once();
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        let (t1, p1) = Task::new(|| {
            return Ok(true);
        });

        let (t2, p2) = Task::new(|| {
            return Ok(false);
        });

        let root_promise = p1.then(|promise_result| {
            match promise_result {
                Ok(task_result) => {
                    match task_result {
                        Ok(value) => {
                            assert_eq!(value, true);
                        }
                        Err(_) => { unreachable!(); }
                    };
                }
                Err(_) => { unreachable!(); }
            };
            Ok(())
        }).join(p2.then(|promise_result| {
            match promise_result {
                Ok(task_result) => {
                    match task_result {
                        Ok(value) => {
                            assert_eq!(value, false);
                        }
                        Err(_) => { unreachable!(); }
                    };
                }
                Err(_) => { unreachable!(); }
            };
            Ok(())
        })).then(|_: Result<((), ()), Canceled>| {
            // Just discard the results we don't really care
            return Ok(());
        });

        sx.send(t1).unwrap();
        sx.send(t2).unwrap();

        tokio::run(root_promise);

        drop(sx);
        g1.join().unwrap();
    }
}
