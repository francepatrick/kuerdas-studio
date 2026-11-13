use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub direction: String,
    pub is_kpt_pro: bool,
    pub configs: Vec<ConfigInfo>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigInfo {
    pub channels: u16,
    pub min_sample_rate: u32,
    pub max_sample_rate: u32,
    pub sample_format: String,
}

// IDs are CPAL host-scoped identifiers, never names or enumeration indices.
pub fn enumerate() -> Result<Vec<DeviceInfo>, String> {
    let host = cpal::default_host();
    let mut result = Vec::new();
    for (direction, devices) in [
        ("input", host.input_devices()),
        ("output", host.output_devices()),
    ] {
        for device in devices.map_err(|e| e.to_string())? {
            let id = device.id().map_err(|e| e.to_string())?.to_string();
            let name = device
                .description()
                .map(|d| d.name().to_owned())
                .unwrap_or_else(|_| "Unnamed device".into());
            let configs: Vec<_> = if direction == "input" {
                device.supported_input_configs().map(|v| v.collect())
            } else {
                device.supported_output_configs().map(|v| v.collect())
            }
            .map_err(|e| e.to_string())?;
            result.push(DeviceInfo {
                is_kpt_pro: is_kpt_pro_identity(&name, &id),
                id,
                name,
                direction: direction.into(),
                configs: configs
                    .into_iter()
                    .map(|c| ConfigInfo {
                        channels: c.channels(),
                        min_sample_rate: c.min_sample_rate(),
                        max_sample_rate: c.max_sample_rate(),
                        sample_format: c.sample_format().to_string(),
                    })
                    .collect(),
            });
        }
    }
    Ok(result)
}
pub fn find(id: &str, input: bool) -> Result<cpal::Device, String> {
    let host = cpal::default_host();
    let devices = if input {
        host.input_devices()
    } else {
        host.output_devices()
    }
    .map_err(|e| e.to_string())?;
    devices
        .into_iter()
        .find(|d| d.id().map(|v| v.to_string() == id).unwrap_or(false))
        .ok_or_else(|| "Selected device is unavailable. Refresh devices and select again.".into())
}

/// Initial detection policy: advertised CoreAudio model name. Replace/augment
/// with verified vendor/product identity when real KPT Pro USB IDs are known.
pub fn is_kpt_pro_name(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    let words: Vec<_> = normalized
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    words.contains(&"kptpro") || words.windows(2).any(|w| w == ["kpt", "pro"])
}
/// The connected KPT Pro advertises Sinco-Audio through Jieli's USB driver.
/// Match both the exact alias and native USB UID lineage, never all USB audio.
pub fn is_kpt_pro_identity(name: &str, id: &str) -> bool {
    if is_kpt_pro_name(name) {
        return true;
    }
    if !name.eq_ignore_ascii_case("Sinco-Audio") {
        return false;
    }
    let Some(identity) =
        id.strip_prefix("coreaudio:AppleUSBAudioEngine:Jieli Technology:USB Composite Device:")
    else {
        return false;
    };
    identity.rsplit_once(':').is_some_and(|(serial, endpoint)| {
        !serial.is_empty() && !endpoint.is_empty() && endpoint.bytes().all(|b| b.is_ascii_digit())
    })
}

pub fn require_kpt_pro(device: &cpal::Device) -> Result<(), String> {
    let description = device.description().map_err(|e| e.to_string())?;
    let id = device.id().map_err(|e| e.to_string())?.to_string();
    if is_kpt_pro_identity(description.name(), &id) {
        Ok(())
    } else {
        Err("Connect KPT Pro and select its audio input to start monitoring. Other input devices are not enabled.".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recognizes_model_name_variants() {
        for name in [
            "KPT Pro",
            "KPT-PRO",
            "KPTPRO",
            "KPT Pro USB Audio",
            "USB KPT_Pro",
        ] {
            assert!(is_kpt_pro_name(name), "{name}");
        }
    }
    #[test]
    fn matches_observed_sinco_alias_only_with_usb_driver_identity() {
        let uid =
            "coreaudio:AppleUSBAudioEngine:Jieli Technology:USB Composite Device:test-serial:2";
        assert!(is_kpt_pro_identity("Sinco-Audio", uid));
        assert!(!is_kpt_pro_identity(
            "Sinco-Audio",
            "coreaudio:BuiltInMicrophoneDevice"
        ));
        assert!(!is_kpt_pro_identity("USB Audio Device", uid));
        assert!(!is_kpt_pro_identity(
            "Sinco-Audio",
            "coreaudio:AppleUSBAudioEngine:Jieli Technology:USB Composite Device::2"
        ));
    }
    #[test]
    fn rejects_other_or_generic_audio_devices() {
        for name in [
            "MacBook Pro Microphone",
            "USB Audio Device",
            "KPT",
            "KPT Professional",
            "MyKPTProInterface",
            "",
        ] {
            assert!(!is_kpt_pro_name(name), "{name}");
        }
    }
}
