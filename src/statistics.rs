use std::{collections::HashMap, fs, io, process::Command, str::FromStr};

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

type Result<T> = std::result::Result<T, Error>;

enum Error {
    FailedToGetBattery,
}

#[derive(Debug)]
pub struct Battery {
    pub charge_now: u64,
    pub charge_full: u64,
    pub percentage: f64,
    pub capacity: u64,
    pub is_charging: bool,
}

impl Battery {
    pub fn new() -> io::Result<Self> {
        const POWER_DIR: &str = "/sys/class/power_supply/";
        const POWER_BATTERY_TYPE: &str = "Battery";
        const POWER_AC_TYPE: &str = "Mains";

        let mut capacity: u64 = 0;
        let mut charge_now: u64 = 0;
        let mut charge_full: u64 = 0;
        let mut percentage: f64 = 0.0;
        let mut is_charging = false;

        let entries = fs::read_dir(POWER_DIR)?;
        for entry in entries {
            let path = entry.unwrap().path();
            let constructed_path = path.to_str().unwrap().to_string() + &"/type".to_string();
            let power_type: io::Result<String> = value_from_file(&constructed_path);

            match power_type.unwrap().as_str() {
                POWER_BATTERY_TYPE => {
                    let path = path.to_str().unwrap().to_string();
                    capacity = value_from_file(&(path.to_string() + &"/capacity".to_string()))?;

                    charge_now = value_from_file(&(path.to_string() + &"/charge_now".to_string()))?;

                    charge_full =
                        value_from_file(&(path.to_string() + &"/charge_full".to_string()))?;

                    percentage = (charge_now as f64 / charge_full as f64) * 100.0;
                }
                POWER_AC_TYPE => {
                    let constructed_path =
                        path.to_str().unwrap().to_string() + &"/online".to_string();
                    let file_value: io::Result<u8> = value_from_file(&constructed_path);
                    is_charging = u8_to_bool(file_value.unwrap());
                }
                _ => {}
            }
        }

        Ok(Battery {
            charge_full,
            capacity,
            charge_now,
            percentage,
            is_charging,
        })
    }
}

#[derive(Debug)]
pub struct Brightness {
    pub current: u32,
    pub max: u32,
    pub percentage: f32,
}

impl Brightness {
    pub fn new() -> io::Result<Brightness> {
        const BRIGHTNESS: &str = "/sys/class/backlight/intel_backlight/brightness";
        const MAX_BRIGHTNESS: &str = "/sys/class/backlight/intel_backlight/max_brightness";
        let current: u32 = value_from_file(&BRIGHTNESS.to_string())?;
        let max: u32 = value_from_file(&MAX_BRIGHTNESS.to_string())?;

        Ok(Brightness {
            current,
            max,
            percentage: (current as f32 / max as f32) * 100.0,
        })
    }
}

pub struct Volume {
    pub volume: u8,
    pub muted: bool,
}
impl Volume {
    pub fn new() -> Self {
        const PROGRAM: &str = "pactl";

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
        let output = Command::new(&PROGRAM)
            .arg("get-sink-mute")
            .arg("@DEFAULT_SINK@")
            .output()
            .expect("Failed to run command");
        let parsed_output = String::from_utf8(output.stdout)
            .expect("Failed to convert command output from utf8 to string");
        let parsed_output: Vec<&str> = parsed_output.split(" ").collect();
        let volume = parsed_output[1].to_string();
        Volume {
            volume: volume.parse::<u8>().expect("Failed to parse volume level"),
            muted: match volume.as_str().trim_end() {
                "yes" => true,
                "no" => false,
                _ => false,
            },
        }
    }
}

pub struct Memory {
    pub free: u32,
    pub used: u32,
    pub available: u32,
    pub total: u32,
}

impl Memory {
    pub fn new() -> io::Result<Self> {
        const MEMORY_PATH: &str = "/proc/meminfo";
        let contents: String = value_from_file(&MEMORY_PATH.to_string())?;

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
            .unwrap_or(0);
        let available: u32 = info
            .get("MemAvailable")
            .expect("Failed to find 'MemFree' in meminfo")
            .to_string()
            .parse()
            .unwrap_or(0);
        let total: u32 = info
            .get("MemTotal")
            .expect("Failed to find 'MemTotal' in meminfo")
            .to_string()
            .parse().unwrap_or(0);

        return Ok(Memory {
            free,
            total,
            available,
            used: total - free,
        })
    }
}
