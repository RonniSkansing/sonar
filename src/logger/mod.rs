#[derive(Debug, PartialEq, Clone)]
pub enum LogLevel {
    NONE,
    NORMAL,
    VERBOSE,
}

#[derive(Debug, Clone)]
pub struct Logger {
    level: LogLevel,
}

impl Logger {
    pub fn new(level: LogLevel) -> Logger {
        Logger { level: level }
    }

    pub fn error(&self, message: String) {
        match self.level {
            LogLevel::NONE => (),
            LogLevel::NORMAL => eprintln!("Error {}", message),
            LogLevel::VERBOSE => eprintln!("Error {}", message),
        }
    }

    pub fn log(&self, message: String) {
        match self.level {
            LogLevel::NONE => (),
            LogLevel::NORMAL => println!("{}", message),
            LogLevel::VERBOSE => println!("{}", message),
        }
    }

    pub fn info(&self, message: String) {
        match self.level {
            LogLevel::NONE => (),
            LogLevel::NORMAL => (),
            LogLevel::VERBOSE => println!("INFO {}", message),
        }
    }
}
