use std::fs::read_to_string;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    wifi_ssid: String,
    wifi_psk: String,
}

fn main() {
    let config: Config = toml::from_str(&read_to_string("cfg.toml").unwrap()).unwrap();

    println!("cargo:rustc-env=CONFIG_WIFI_SSID={}", config.wifi_ssid);
    println!("cargo:rustc-env=CONFIG_WIFI_PSK={}", config.wifi_psk);

    embuild::espidf::sysenv::output();
}
