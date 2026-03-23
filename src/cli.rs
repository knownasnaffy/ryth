use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "ryth", about = "Scriptable iwd interface")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Current wifi status (state, signal, powered, etc.)
    Status {
        /// Stream updates on change
        #[arg(long)]
        watch: bool,
    },
    /// List discovered networks (cached)
    List {
        /// Re-output on each scan cycle
        #[arg(long)]
        watch: bool,
    },
    /// Trigger a scan and return updated network list
    Scan,
    /// Connect to a network by SSID
    Connect {
        ssid: String,
        #[arg(long, value_name = "PASSWORD")]
        password: Option<String>,
    },
    /// Disconnect from the current network
    Disconnect,
    /// Toggle autoconnect for a known network
    Autoconnect {
        ssid: String,
        state: OnOff,
    },
    /// Forget a known network
    Forget { ssid: String },
    /// List known networks
    Known,
    /// Toggle wifi power
    Power { state: OnOff },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OnOff {
    On,
    Off,
}

impl From<OnOff> for bool {
    fn from(v: OnOff) -> bool {
        matches!(v, OnOff::On)
    }
}
