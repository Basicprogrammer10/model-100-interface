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

#[macro_export]
macro_rules! concat_arrays_size {
    ($( $array:expr ),*) => {{
        0 $(+ $array.len())*
    }};
}

/// Modified from the [array_concat](https://crates.io/crates/array-concat) crate.
#[macro_export]
macro_rules! join_arrays {
    ($($array:expr),*) => ({
        #[repr(C)]
        struct ArrayConcatDecomposed<T>($([T; $array.len()]),*);

        #[repr(C)]
        union ArrayConcatComposed<T, const N: usize> {
            full: core::mem::ManuallyDrop<[T; N]>,
            decomposed: core::mem::ManuallyDrop<ArrayConcatDecomposed<T>>,
        }

        const SIZE: usize = $crate::concat_arrays_size!($($array),*);
        let composed = ArrayConcatComposed::<_, SIZE> {
            decomposed: core::mem::ManuallyDrop::new(ArrayConcatDecomposed($($array),*))
        };

        core::mem::ManuallyDrop::into_inner(unsafe { composed.full })
    })
}
