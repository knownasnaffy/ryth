use serde::Serialize;

/// Convert iwd signal strength (dBm × 100) to 0–100 percentage.
/// iwd reports values like -4200 meaning -42.00 dBm.
/// Range: -30 dBm (100%) to -90 dBm (0%).
pub fn signal_to_strength(raw: i16) -> u8 {
    let dbm = raw as f32 / 100.0;
    let clamped = dbm.clamp(-90.0, -30.0);
    ((clamped + 90.0) / 60.0 * 100.0).round() as u8
}

#[derive(Debug, Serialize)]
pub struct Status {
    pub powered: bool,
    pub state: String,
    pub ssid: Option<String>,
    pub strength: Option<u8>,
}

#[derive(Debug, Serialize)]
pub struct NetworkEntry {
    pub ssid: String,
    pub strength: u8,
    #[serde(rename = "type")]
    pub network_type: String,
    pub known: bool,
    pub connected: bool,
}

#[derive(Debug, Serialize)]
pub struct KnownEntry {
    pub ssid: String,
    #[serde(rename = "type")]
    pub network_type: String,
    pub autoconnect: bool,
    pub last_connected: Option<String>,
}
