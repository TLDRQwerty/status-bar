use std::fs;
use std::io;
use std::process::Command;
use std::str::FromStr;
use std::thread;
use std::time;

fn value_from_file<T: FromStr>(path: &String) -> io::Result<T> {
    fs::read_to_string(path)
        .unwrap_or(format!("Failed to find file {}", &path))
        .trim_end_matches("\n")
        .parse()
        .and_then(|v: T| Ok(v))
        .or_else(|_| {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to find file {}", &path),
            ))
        })
}

fn u8_to_bool(v: u8) -> bool {
    match v {
        1 => true,
        _ => false,
    }
}

pub mod statistics {
    pub mod battery {
        pub const POWER_DIR: &str = "/sys/class/power_supply/";
        pub const POWER_BATTERY_TYPE: &str = "Battery";
        pub const POWER_AC_TYPE: &str = "Mains";
        use std::{fs, io};

        use crate::{u8_to_bool, value_from_file};

        pub fn read_capacity() -> u8 {
            let entries = fs::read_dir(POWER_DIR).expect("Failed to read dir");
            let mut capacity: u8 = 0;
            for entry in entries {
                let path = entry.unwrap().path();
                let constructed_path = path.to_str().unwrap().to_string() + &"/type".to_string();
                let power_type: io::Result<String> = value_from_file(&constructed_path);

                match power_type.unwrap().as_str() {
                    POWER_BATTERY_TYPE => {
                        let path = path.to_str().unwrap().to_string();
                        capacity = value_from_file(&(path.to_string() + &"/capacity".to_string()))
                            .expect("Failed to get charge left");
                        break;
                    }
                    POWER_AC_TYPE => {}
                    _ => panic!("Unknown battery type"),
                }
            }

            capacity
        }

        pub fn read_remaning_charge() -> f64 {
            let entries = fs::read_dir(POWER_DIR).expect("Failed to read dir");
            let mut remaining: f64 = 0.0;
            for entry in entries {
                let path = entry.unwrap().path();
                let constructed_path = path.to_str().unwrap().to_string() + &"/type".to_string();
                let power_type: io::Result<String> = value_from_file(&constructed_path);

                match power_type.unwrap().as_str() {
                    POWER_BATTERY_TYPE => {
                        let path = path.to_str().unwrap().to_string();
                        let charge_now: f64 =
                            value_from_file(&(path.to_string() + &"/charge_now".to_string()))
                                .expect("Failed to get charge left");
                        let charge_full: f64 =
                            value_from_file(&(path.to_string() + &"/charge_full".to_string()))
                                .expect("Failed to get charge left");
                        remaining = (charge_now / charge_full) * 100.0;
                        break;
                    }
                    POWER_AC_TYPE => {}
                    _ => panic!("Unknown battery type"),
                }
            }

            remaining
        }

        pub fn is_charging() -> bool {
            let entries = match fs::read_dir(POWER_DIR) {
                Ok(r) => r,
                Err(e) => panic!("{}", e),
            };

            let mut charging = false;
            for entry in entries {
                let path = entry.unwrap().path();
                let constructed_path = path.to_str().unwrap().to_string() + &"/type".to_string();
                let power_type: io::Result<String> = value_from_file(&constructed_path);
                match power_type.unwrap().as_str() {
                    POWER_AC_TYPE => {
                        let constructed_path =
                            path.to_str().unwrap().to_string() + &"/online".to_string();
                        let file_value: io::Result<u8> = value_from_file(&constructed_path);
                        let is_charging = u8_to_bool(file_value.unwrap());
                        charging = is_charging;
                        break;
                    }
                    _ => {}
                }
            }

            charging
        }
    }

    pub mod memory {
        use std::collections::HashMap;

        use crate::value_from_file;

        const MEMORY_PATH: &str = "/proc/meminfo";

        pub fn usage() -> HashMap<String, String> {
            let contents: String =
                value_from_file(&MEMORY_PATH.to_string()).expect("Failed to get memory info");

            let mut info: HashMap<String, String> = HashMap::new();

            let values: Vec<&str> = contents.split("\n").collect();
            for entry in values {
                let v: Vec<&str> = entry.split(":").collect();
                let mem: Vec<&str> = v[1].split_whitespace().collect();
                info.insert(
                    v[0].to_string().trim_end().to_string(),
                    mem[0].to_string().trim_start().to_string(),
                );
            }

            return info;
        }
    }
}

fn date() -> String {
    chrono::Local::now()
        .format("%H:%M:%S | %d/%m/%y")
        .to_string()
}

fn main() {
    loop {
        let remaining_charge = statistics::battery::read_remaning_charge();
        let memory = statistics::memory::usage();
        let memory_free = &memory.get("MemFree").expect("Failed").to_string();
        let memory_total = &memory.get("MemTotal").expect("Failed").to_string();
        let _ = Command::new("xsetroot")
            .arg("-name")
            .arg(
                "B ".to_string()
                    + &format!("{:.1}", remaining_charge)
                    + &" | "
                    + &"M ".to_string()
                    + &format!("{} / {}", memory_free, memory_total)
                    + &" | "
                    + &format!("{}", date()),
            )
            .output();
        thread::sleep(time::Duration::from_secs(2));
    }
}
