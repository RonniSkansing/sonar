#[derive(Debug, PartialEq, Clone)]
pub enum LogLevel {
    NONE,
    NORMAL,
    VERBOSE,
}

impl LogLevel {
    pub fn from_string(s: &str) -> LogLevel {
        match s {
            "NONE" => LogLevel::NONE,
            "NORMAL" => LogLevel::NORMAL,
            "VERBOSE" => LogLevel::VERBOSE,
            _ => panic!("Unknown log level {}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Logger {
    level: LogLevel,
}

impl Logger {
    pub fn new(level: LogLevel) -> Logger {
        Logger { level: level }
    }

    /*
    pub fn error(&self, message: &String) {
        match self.level {
            LogLevel::NONE => (),
            LogLevel::NORMAL => println!("{}", message),
            LogLevel::VERBOSE => println!("{}", message),
        }
    }
    */

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
            LogLevel::VERBOSE => println!("{}", message),
        }
    }
}
