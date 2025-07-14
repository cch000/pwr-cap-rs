mod profile;
mod system;

use profile::Profile;
use system::{
    PowerProfileValue::{Balaced, Performance, Quiet},
    System,
};

use libryzenadj::RyzenAdj;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fs;
use std::thread::sleep;
use std::time::{self, Duration};

const NAP_TIME: Duration = time::Duration::from_secs(10);

#[cfg(debug_assertions)]
const CONFIG_PATH: &str = "./example-config/ryzencap.json";
#[cfg(not(debug_assertions))]
const CONFIG_PATH: &str = "/etc/ryzencap.json";

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
    fn load() -> Result<Self> {
        let buffer = fs::read_to_string(CONFIG_PATH).expect("Failed to load config");
        serde_json::from_str(&buffer)
    }
}

fn main() {
    let config: Config = Config::load().unwrap();
    let ryzenadj: RyzenAdj = RyzenAdj::new().unwrap();
    loop {
        let system: System = System::new();

        match system.get_power_profile() {
            Quiet => {
                if system.get_connected() {
                    config.quiet.plugged.apply(&ryzenadj);
                } else {
                    config.quiet.unplugged.apply(&ryzenadj);
                }
            }
            Balaced => {
                if system.get_connected() {
                    config.balanced.plugged.apply(&ryzenadj);
                } else {
                    config.balanced.unplugged.apply(&ryzenadj);
                }
            }
            Performance => {
                if system.get_connected() {
                    config.performance.plugged.apply(&ryzenadj);
                } else {
                    config.performance.unplugged.apply(&ryzenadj);
                }
            }
        }

        // tctl limit is a global config, for all profiles
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
