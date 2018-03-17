use std::error::Error;
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum TaskErrorCode {
    Unknown,
    Panic,
}

#[derive(Debug)]
pub struct TaskError {
    message: String,
    code: TaskErrorCode,
}

impl TaskError {
    fn message_for(code: TaskErrorCode, message: Option<&str>) -> String {
        match message {
            Some(m) => format!("{:?}: {:?}", code, m),
            None => format!("{:?}", code)
        }
    }
}

impl Error for TaskError {
    fn description(&self) -> &str {
        return &self.message;
    }
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.description())
    }
}

impl From<TaskErrorCode> for TaskError {
    fn from(code: TaskErrorCode) -> Self {
        return TaskError {
            code,
            message: TaskError::message_for(code, None),
        };
    }
}

impl<'a> From<&'a str> for TaskError {
    fn from(message: &'a str) -> Self {
        return TaskError {
            code: TaskErrorCode::Unknown,
            message: TaskError::message_for(TaskErrorCode::Unknown, Some(message)),
        };
    }
}