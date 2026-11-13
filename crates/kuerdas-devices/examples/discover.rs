use cpal::traits::{DeviceTrait, HostTrait};
fn main() {
    for device in kuerdas_devices::enumerate().expect("Kuerdas discovery failed") {
        println!(
            "Kuerdas match: {} / {} / KPT Pro = {}",
            device.name, device.direction, device.is_kpt_pro
        );
    }
    let host = cpal::default_host();
    for device in host.devices().expect("device discovery failed") {
        println!("Device: {:?}", device.description());
        println!("ID: {:?}", device.id().map(|id| id.to_string()));
        println!(
            "Input configs: {:?}",
            device
                .supported_input_configs()
                .map(|c| c.collect::<Vec<_>>())
        );
        println!(
            "Output configs: {:?}",
            device
                .supported_output_configs()
                .map(|c| c.collect::<Vec<_>>())
        );
    }
}
