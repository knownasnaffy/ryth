use anyhow::{Context, Result, bail};
use futures_lite::stream::StreamExt;
use iwdrs::{agent::Agent, network::Network, session::Session, station::State};

use ryth::output::{KnownEntry, NetworkEntry, Status, signal_to_strength};

// ── helpers ──────────────────────────────────────────────────────────────────

async fn get_station(session: &Session) -> Result<iwdrs::station::Station> {
    session
        .stations()
        .await?
        .into_iter()
        .next()
        .context("no station found")
}

async fn get_device(session: &Session) -> Result<iwdrs::device::Device> {
    session
        .devices()
        .await?
        .into_iter()
        .next()
        .context("no device found")
}

async fn network_entries(station: &iwdrs::station::Station) -> Result<Vec<NetworkEntry>> {
    let networks = station.discovered_networks().await?;
    let mut entries = Vec::with_capacity(networks.len());
    for (net, signal) in &networks {
        entries.push(NetworkEntry {
            ssid: net.name().await?,
            strength: signal_to_strength(*signal),
            network_type: net.network_type().await?.to_string(),
            known: net.known_network().await?.is_some(),
            connected: net.connected().await?,
        });
    }
    Ok(entries)
}

async fn build_status(session: &Session) -> Result<Status> {
    let device = get_device(session).await?;
    let powered = device.is_powered().await?;
    let station = get_station(session).await?;
    let state = station.state().await?;

    let (ssid, strength) = if matches!(state, State::Connected) {
        let networks = station.discovered_networks().await?;
        let mut found_ssid = None;
        let mut found_strength = None;
        for (net, sig) in &networks {
            if net.connected().await.unwrap_or(false) {
                found_ssid = net.name().await.ok();
                found_strength = Some(signal_to_strength(*sig));
                break;
            }
        }
        (found_ssid, found_strength)
    } else {
        (None, None)
    };

    Ok(Status {
        powered,
        state: state.to_string().to_lowercase(),
        ssid,
        strength,
    })
}

// ── commands ─────────────────────────────────────────────────────────────────

pub async fn status(watch: bool) -> Result<()> {
    let session = Session::new().await?;
    if !watch {
        let s = build_status(&session).await?;
        println!("{}", serde_json::to_string(&s)?);
        return Ok(());
    }

    let station = get_station(&session).await?;
    let mut state_stream = station.state_stream().await?;
    while let Some(state_res) = state_stream.next().await {
        let _ = state_res?;
        let session2 = Session::new().await?;
        let s = build_status(&session2).await?;
        println!("{}", serde_json::to_string(&s)?);
    }
    Ok(())
}

pub async fn list(watch: bool) -> Result<()> {
    let session = Session::new().await?;
    let station = get_station(&session).await?;

    if !watch {
        let entries = network_entries(&station).await?;
        println!("{}", serde_json::to_string(&entries)?);
        return Ok(());
    }

    loop {
        let entries = network_entries(&station).await?;
        println!("{}", serde_json::to_string(&entries)?);
        // wait for a full scan cycle: Scanning=true then Scanning=false
        station.scan().await.ok(); // trigger if not already scanning; ignore error if busy
        station.wait_for_scan_complete().await?;
    }
}

pub async fn scan() -> Result<()> {
    let session = Session::new().await?;
    let station = get_station(&session).await?;
    station.scan().await?;
    station.wait_for_scan_complete().await?;
    let entries = network_entries(&station).await?;
    println!("{}", serde_json::to_string(&entries)?);
    Ok(())
}

pub async fn connect(ssid: String, password: Option<String>) -> Result<()> {
    let session = Session::new().await?;
    let agent = PasswordAgent(password);
    let _agent_manager = session.register_agent(agent).await?;
    let station = get_station(&session).await?;

    let networks = station.discovered_networks().await?;
    let network = find_network(&ssid, &networks).await;

    let network = match network {
        Some(n) => n,
        None => {
            station.scan().await?;
            station.wait_for_scan_complete().await?;
            let networks = station.discovered_networks().await?;
            find_network(&ssid, &networks)
                .await
                .with_context(|| format!("network '{ssid}' not found"))?
        }
    };

    network.connect().await?;
    Ok(())
}

pub async fn disconnect() -> Result<()> {
    let session = Session::new().await?;
    let station = get_station(&session).await?;
    station.disconnect().await?;
    Ok(())
}

pub async fn autoconnect(ssid: String, enabled: bool) -> Result<()> {
    let session = Session::new().await?;
    for kn in session.known_networks().await? {
        if kn.name().await? == ssid {
            kn.set_autoconnect(enabled).await?;
            return Ok(());
        }
    }
    bail!("known network '{ssid}' not found");
}

pub async fn forget(ssid: String) -> Result<()> {
    let session = Session::new().await?;
    for kn in session.known_networks().await? {
        if kn.name().await? == ssid {
            kn.forget().await?;
            return Ok(());
        }
    }
    bail!("known network '{ssid}' not found");
}

pub async fn known() -> Result<()> {
    let session = Session::new().await?;
    let mut entries = vec![];
    for kn in session.known_networks().await? {
        entries.push(KnownEntry {
            ssid: kn.name().await?,
            network_type: kn.network_type().await?.to_string(),
            autoconnect: kn.get_autoconnect().await?,
            last_connected: kn.last_connected_time().await.ok(),
        });
    }
    println!("{}", serde_json::to_string(&entries)?);
    Ok(())
}

pub async fn power(on: bool) -> Result<()> {
    let session = Session::new().await?;
    let device = get_device(&session).await?;
    device.set_power(on).await?;
    Ok(())
}

// ── agent ────────────────────────────────────────────────────────────────────

struct PasswordAgent(Option<String>);

impl Agent for PasswordAgent {
    fn request_passphrase(
        &self,
        _network: &Network,
    ) -> impl Future<Output = Result<String, iwdrs::error::agent::Canceled>> + Send {
        std::future::ready(match self.0.as_ref() {
            Some(pw) => Ok(pw.clone()),
            None => Err(iwdrs::error::agent::Canceled {}),
        })
    }

    fn request_private_key_passphrase(
        &self,
        _network: &Network,
    ) -> impl Future<Output = Result<String, iwdrs::error::agent::Canceled>> + Send {
        std::future::ready(Err(iwdrs::error::agent::Canceled {}))
    }

    fn request_user_name_and_passphrase(
        &self,
        _network: &Network,
    ) -> impl Future<Output = Result<(String, String), iwdrs::error::agent::Canceled>> + Send {
        std::future::ready(Err(iwdrs::error::agent::Canceled {}))
    }

    fn request_user_password(
        &self,
        _network: &Network,
        _user_name: Option<&String>,
    ) -> impl Future<Output = Result<String, iwdrs::error::agent::Canceled>> + Send {
        std::future::ready(Err(iwdrs::error::agent::Canceled {}))
    }
}

// ── network lookup ────────────────────────────────────────────────────────────

async fn find_network(ssid: &str, networks: &[(Network, i16)]) -> Option<Network> {
    for (net, _) in networks {
        if net.name().await.ok().as_deref() == Some(ssid) {
            return Some(net.clone());
        }
    }
    None
}
