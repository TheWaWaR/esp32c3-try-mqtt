use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

// Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
fn main() -> Result<(), Box<dyn std::error::Error>> {
    embuild::build::CfgArgs::output_propagated("ESP_IDF")?;
    embuild::build::LinkArgs::output_propagated("ESP_IDF")?;

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("envs.rs");
    let mut out_file = BufWriter::new(File::create(&out_path).expect("create envs.rs"));
    writeln!(
        &mut out_file,
        "pub const WIFI_SSID: &str = \"{}\";",
        env!("ESP_WIFI_SSID"),
    )
    .expect("write to envs.rs");
    writeln!(
        &mut out_file,
        "pub const WIFI_PASS: &str = \"{}\";",
        env!("ESP_WIFI_PASS"),
    )
    .expect("write to envs.rs");

    writeln!(
        &mut out_file,
        "pub const MQTT_HOST: &str = \"{}\";",
        env!("ESP_MQTT_HOST"),
    )
    .expect("write to envs.rs");
    writeln!(
        &mut out_file,
        "pub const MQTT_USER: &str = \"{}\";",
        env!("ESP_MQTT_USER"),
    )
    .expect("write to envs.rs");
    writeln!(
        &mut out_file,
        "pub const MQTT_PASS: &str = \"{}\";",
        env!("ESP_MQTT_PASS"),
    )
    .expect("write to envs.rs");

    Ok(())
}
