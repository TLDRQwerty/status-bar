use std::process::Command;
use std::thread;
use std::time;

use status_bar::statistics::*;

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
                    + &format!("B{} {:.1}", is_charging, remaining_charge.percentage)
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
