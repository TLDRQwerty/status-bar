use std::process::Command;
use std::thread;
use std::time;

use status_bar::statistics::{Battery, Brightness, Memory, Volume};

const SEPERATOR: &str = "|";

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
        let date = date();
        let battery = Battery::new();
        let volume = Volume::new();
        let memory = Memory::new();
        let brightness = Brightness::new();

        let _ = Command::new("xsetroot")
            .arg("-name")
            .arg(&format!("{} {} {}", &date.time, &SEPERATOR, &date.date))
            .output();
        thread::sleep(time::Duration::from_secs(1));
    }
}
