use std::{ collections::VecDeque, sync::{ mpsc::{channel, Receiver, Sender}, Arc, Mutex } };

const STATUS_CAPACITY : usize = 64;

pub struct ConsoleReceiver {
    logs: VecDeque<String>,

    receiver: Arc<Mutex<Receiver<ConsoleLog>>>,
    sender: Sender<ConsoleLog>,

    log_counter: u32,
    log_string: String
}

#[derive(Clone)]
pub struct ConsoleSender {
    sender: Sender<ConsoleLog>
}

pub struct ConsoleLog {
    pub message: String
}

impl ConsoleReceiver {
    pub fn new() -> ConsoleReceiver {
        let (sender, receiver) = channel::<ConsoleLog>();

        let console = Self {
            logs: VecDeque::new(),

            receiver: Arc::from(Mutex::from(receiver)),
            sender: sender,

            log_counter: 0,
            log_string: String::new()
        };

        return console;
    }

    pub fn create_sender(&self) -> ConsoleSender {
        let console_sender = ConsoleSender {
            sender: self.sender.clone()
        };

        return console_sender;
    }

    pub fn get_log_string(&mut self) -> String {
        let updated = self.update();

        if !updated {
            self.log_string = String::new();
            for log in &self.logs {
                self.log_string += log;
            }
    
            self.log_string = String::from(self.log_string.trim_end());
        }

        return self.log_string.clone();
    }

    pub fn get_last_log(&mut self) -> String {
        let _ = self.update();

        return match self.logs.front() {
            Some(l) => String::from(l),
            None => String::new()
        };
    }

    pub fn log(&mut self, message: String) {
        let log = ConsoleLog::new(message);
        self.add_log(log);
    }

    pub fn update(&mut self) -> bool {
        let receiver = self.receiver.clone();
        let receiver_lock = receiver.lock().unwrap();

        let mut updated = false;
        for log in receiver_lock.try_iter() {
            self.add_log(log);
            updated = true;
        }

        return updated;
    }

    fn add_log(&mut self, log: ConsoleLog) {
        let log_string = format!("[{count:04}] {message}\n", count = self.log_counter, message = log.message);
        print!("{}", log_string);
        self.logs.push_front(log_string);
        self.log_counter += 1;

        if self.logs.len() <= STATUS_CAPACITY { return; }

        self.logs.pop_back();
    }
}

impl ConsoleSender {
    pub fn log(&self, message: String) {
        let log = ConsoleLog::new(message);
        self.send_log(log);
    }

    fn send_log(&self, log: ConsoleLog) {
        let _ = self.sender.send(log);
    }
}

impl ConsoleLog {
    pub fn new(message: String) -> ConsoleLog {
        Self {
            message: message
        }
    }
}