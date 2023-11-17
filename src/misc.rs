use anyhow::{Context, Result};
use cpal::{traits::DeviceTrait, Device, Devices, InputDevices};
use num_traits::Num;

pub fn plural(n: impl Num) -> &'static str {
    if n.is_one() {
        ""
    } else {
        "s"
    }
}

pub fn audio_dev(mut devices: InputDevices<Devices>, search: &str) -> Result<Device> {
    let mut best = devices.next().context("No audio devices")?;
    let mut best_similarity = 0.0;

    for device in devices {
        let name = device.name()?;
        let similarity = strsim::sorensen_dice(&name, search);
        if similarity > best_similarity {
            best = device;
            best_similarity = similarity;
        }
    }

    Ok(best)
}
