use std::time::{Duration, SystemTime};
use std::thread::sleep;
use httpdate::HttpDate;
use mosquitto_client::Mosquitto;

pub struct Config {
    pub verbose: u8,
    pub broker: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub port: u16,
    pub updates: usize,
    pub topic: String,
    pub id: String,
}

impl Config {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&self) -> Result<(), String> {
        let mosquitto = Mosquitto::new(&self.id);
        if let Err(err) = mosquitto.connect(&self.broker, self.port as u32) {
            return Err(err.to_string());
        }
        

        loop {
            let our_mid = mosquitto.publish(&self.topic, HttpDate::from(SystemTime::now()).to_string().as_bytes(), 2, false);
            if let Err(err) = our_mid {
                return Err(err.to_string());
            }
            let our_mid = our_mid.unwrap();

            // and wait for confirmation for that message id
            let mut mc = mosquitto.callbacks(());
            mc.on_publish(|_,mid| {
                if mid == our_mid {
                    println!("written");
                }
            });

            sleep(Duration::from_secs(self.updates as u64));
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbose: 0,
            broker: String::from("localhost"),
            username: None,
            password: None,
            port: 1883,
            updates: 5,
            topic: String::from("time"),
            id: String::from(env!("CARGO_PKG_NAME")),
        }
    }
}
