use std::fs;
use std::io;
use std::process::Command;
use std::str::FromStr;
use std::thread;
use std::time;

const SEPERATOR: &str = "|";

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

    pub mod brightness {
        use crate::value_from_file;

        const BRIGHTNESS: &str = "/sys/class/backlight/intel_backlight/brightness";
        const MAX_BRIGHTNESS: &str = "/sys/class/backlight/intel_backlight/max_brightness";

        #[derive(Debug)]
        pub struct Brightness {
            pub current: u32,
            pub max: u32,
            pub percentage: f32,
        }

        pub fn brightness() -> Brightness {
            let current: u32 = value_from_file(&BRIGHTNESS.to_string())
                .expect("Failed to get and parse current backlight value");
            let max: u32 = value_from_file(&MAX_BRIGHTNESS.to_string())
                .expect("Failed to get and parse max backlight value");

            Brightness {
                current,
                max,
                percentage: (current as f32 / max as f32) * 100.0,
            }
        }
    }

    pub mod volume {
        use std::process::Command;

        const PROGRAM: &str = "pactl";

        pub fn get_volume() -> u8 {
            let output = Command::new(&PROGRAM)
                .arg("get-sink-volume")
                .arg("@DEFAULT_SINK@")
                .output()
                .expect("Failed to run command");
            let parsed_output = String::from_utf8(output.stdout)
                .expect("Failed to convert command output from utf8 to string");
            let parsed_output: Vec<&str> = parsed_output.split(" ").collect();
            let mut volume = parsed_output[5].to_string();
            let _ = &volume.pop().expect("Failed to pop last value");
            volume.parse::<u8>().expect("Failed to parse volume level")
        }

        pub fn is_muted() -> bool {
            let output = Command::new(&PROGRAM)
                .arg("get-sink-mute")
                .arg("@DEFAULT_SINK@")
                .output()
                .expect("Failed to run command");
            let parsed_output = String::from_utf8(output.stdout)
                .expect("Failed to convert command output from utf8 to string");
            let parsed_output: Vec<&str> = parsed_output.split(" ").collect();
            let volume = parsed_output[1].to_string();
            match volume.as_str().trim_end() {
                "yes" => true,
                "no" => false,
                _ => panic!("Could not workout if volume is muted or not"),
            }
        }
    }

    pub mod memory {
        use std::collections::HashMap;

        use crate::value_from_file;

        const MEMORY_PATH: &str = "/proc/meminfo";

        pub struct Memory {
            pub free: u32,
            pub used: u32,
            pub total: u32,
        }

        pub fn usage() -> Memory {
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

            let free: u32 = info
                .get("MemFree")
                .expect("Failed to find 'MemFree' in meminfo")
                .to_string()
                .parse()
                .expect("Failed to parse memory free");
            let total: u32 = info
                .get("MemTotal")
                .expect("Failed to find 'MemTotal' in meminfo")
                .to_string()
                .parse()
                .expect("Failed to parse memory used");
            return Memory {
                free,
                total,
                used: total - free,
            };
        }
    }
}

struct Date {
    date: String,
    time: String,
}

fn date() -> Date {
    Date {
        date: chrono::Local::now().format("%d/%m/%y").to_string(),
        time: chrono::Local::now().format("%H:%M:%S").to_string(),
    }
}

fn battery_is_charging(charging: bool) -> String {
    match charging {
        true => "+".to_string(),
        false => "-".to_string(),
    }
}

fn volume_is_muted(muted: bool) -> String {
    match muted {
        true => "(x)".to_string(),
        _ => "".to_string(),
    }
}

fn main() {
    loop {
        let remaining_charge = statistics::battery::read_remaning_charge();
        let is_charging = battery_is_charging(statistics::battery::is_charging());
        let memory = statistics::memory::usage();
        let brightness = statistics::brightness::brightness();
        let volume = statistics::volume::get_volume();
        let muted = statistics::volume::is_muted();
        let date = date();
        let _ = Command::new("xsetroot")
            .arg("-name")
            .arg(
                format!("V{} {:.}% ", volume_is_muted(muted), volume)
                    + &format!(" {} ", &SEPERATOR)
                    + &format!("b {:.}% ", brightness.percentage)
                    + &format!(" {} ", &SEPERATOR)
                    + &format!("B{} {:.1}", is_charging, remaining_charge)
                    + &format!(" {} ", &SEPERATOR)
                    + &format!(
                        "M {} / {}",
                        &memory.used.to_string(),
                        &memory.total.to_string()
                    )
                    + &format!(" {} ", &SEPERATOR)
                    + &format!("{} {} {}", &date.time, &SEPERATOR, &date.date),
            )
            .output();
        thread::sleep(time::Duration::from_secs(1));
    }
}
