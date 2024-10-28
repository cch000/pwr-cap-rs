use libryzenadj::RyzenAdj;
use std::fs;
use std::thread::sleep;
use std::time::{self, Duration};

use serde::{Deserialize, Serialize};
use serde_json::Result;

const NAP_TIME: Duration = time::Duration::from_secs(10);
const SYS_POWER_PROFILE: &str = "/sys/firmware/acpi/platform_profile";
const SYS_CONNECTED: &str = "/sys/class/power_supply/AC0/online";

#[cfg(debug_assertions)]
const CONFIG_PATH: &str = "./example-config/pwr-cap-rs.json";
#[cfg(not(debug_assertions))]
const CONFIG_PATH: &str = "/etc/pwr-cap-rs.json";

#[derive(Serialize, Deserialize)]
struct Profile {
    enable: bool,
    stapm_limit: Option<u32>,    // Sustained Power Limit (mW)
    fast_limit: Option<u32>,     // ACTUAL Power Limit    (mW)
    slow_limit: Option<u32>,     // Average Power Limit   (mW)
    apu_slow_limit: Option<u32>, // APU Power Limit       (mW)
}

impl Profile {
    pub fn apply(&self, ryzenadj: &RyzenAdj) {
        if self.enable {
            let fast_limit = ryzenadj.get_fast_limit().unwrap() as u32 * 1000;
            if fast_limit != self.fast_limit.expect("fast limit cannot be null") {
                ryzenadj
                    .set_fast_limit(self.fast_limit.unwrap())
                    .expect("failed to apply fast_limit");
                if self.stapm_limit.is_some() {
                    ryzenadj
                        .set_stapm_limit(self.stapm_limit.unwrap())
                        .expect("failed to apply stapm_limit");
                }

                if self.slow_limit.is_some() {
                    ryzenadj
                        .set_slow_limit(self.slow_limit.unwrap())
                        .expect("failed to apply slow_limit");
                }
                if self.apu_slow_limit.is_some() {
                    ryzenadj
                        .set_apu_slow_limit(self.apu_slow_limit.unwrap())
                        .expect("failed to APU slow limit");
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct QuietProfile {
    plugged: Profile,
    unplugged: Profile,
}

#[derive(Serialize, Deserialize)]
struct BalacedProfile {
    plugged: Profile,
    unplugged: Profile,
}

#[derive(Serialize, Deserialize)]
struct PerformanceProfile {
    plugged: Profile,
    unplugged: Profile,
}

#[derive(Serialize, Deserialize)]
struct Config {
    quiet: QuietProfile,
    balanced: BalacedProfile,
    performance: PerformanceProfile,
    tctl_limit: Option<u32>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let buffer = fs::read_to_string(CONFIG_PATH).expect("Failed to load config");
        Ok(serde_json::from_str(&buffer)?)
    }
}

enum PowerProfileValue {
    Quiet,
    Balaced,
    Performance,
}

struct System {
    power_profile: PowerProfileValue,
    plugged: bool,
}
impl System {
    pub fn new() -> Self {
        System {
            power_profile: System::get_power_profile(),
            plugged: System::get_connected(),
        }
    }
    fn get_power_profile() -> PowerProfileValue {
        match fs::read_to_string(SYS_POWER_PROFILE)
            .expect("Reading pwr profile failed")
            .trim()
        {
            "quiet" => PowerProfileValue::Quiet,
            "balanced" => PowerProfileValue::Balaced,
            "performance" => PowerProfileValue::Performance,
            _ => panic!("power profile not valid"),
        }
    }

    fn get_connected() -> bool {
        if fs::read_to_string(SYS_CONNECTED)
            .expect("Reading plugged status failed")
            .trim()
            == "1"
        {
            true
        } else {
            false
        }
    }
}

fn main() {
    let config: Config = Config::load().unwrap();
    let ryzenadj: RyzenAdj = RyzenAdj::new().unwrap();
    loop {
        let system: System = System::new();

        match system.power_profile {
            PowerProfileValue::Quiet => {
                if system.plugged {
                    config.quiet.plugged.apply(&ryzenadj);
                } else {
                    config.quiet.unplugged.apply(&ryzenadj);
                }
            }
            PowerProfileValue::Balaced => {
                if system.plugged {
                    config.balanced.plugged.apply(&ryzenadj);
                } else {
                    config.balanced.unplugged.apply(&ryzenadj);
                }
            }
            PowerProfileValue::Performance => {
                if system.plugged {
                    config.performance.plugged.apply(&ryzenadj);
                } else {
                    config.performance.unplugged.apply(&ryzenadj);
                }
            }
        }
        if config.tctl_limit.is_some() {
            ryzenadj
                .set_tctl_temp(config.tctl_limit.unwrap())
                .expect("failed to apply tctl limit");
        }

        ryzenadj
            .refresh()
            .expect("failed to refresh ryzenadj values");

        sleep(NAP_TIME);
    }
}
