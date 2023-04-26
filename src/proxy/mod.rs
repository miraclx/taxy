use self::{tcp::TcpPortContext, tls::TlsState};
use crate::{
    config::{port::PortEntry, AppConfig},
    error::Error,
    keyring::Keyring,
};
use serde_derive::Serialize;
use std::time::SystemTime;

pub mod tcp;
pub mod tls;

const MAX_NAME_LEN: usize = 32;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SocketState {
    Listening,
    PortAlreadyInUse,
    PermissionDenied,
    AddressNotAvailable,
    Error,
    #[default]
    Unknown,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PortStatus {
    pub state: PortState,
    #[serde(serialize_with = "serialize_started_at")]
    pub started_at: Option<SystemTime>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PortState {
    pub socket: SocketState,
    pub tls: Option<TlsState>,
}

fn serialize_started_at<S>(
    started_at: &Option<SystemTime>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(started_at) = started_at {
        let started_at = started_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        serializer.serialize_some(&started_at)
    } else {
        serializer.serialize_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortContextEvent {
    SokcetStateUpadted(SocketState),
}

#[derive(Debug)]
pub struct PortContext {
    entry: PortEntry,
    kind: PortContextKind,
}

impl PortContext {
    pub fn new(entry: PortEntry) -> Result<Self, Error> {
        if entry.name.is_empty() || entry.name.len() > MAX_NAME_LEN {
            return Err(Error::InvalidName { name: entry.name });
        }
        let kind = PortContextKind::Tcp(TcpPortContext::new(&entry)?);
        Ok(Self { entry, kind })
    }

    pub fn entry(&self) -> &PortEntry {
        &self.entry
    }

    pub fn kind(&self) -> &PortContextKind {
        &self.kind
    }

    pub fn kind_mut(&mut self) -> &mut PortContextKind {
        &mut self.kind
    }

    pub async fn prepare(&mut self, config: &AppConfig) -> Result<(), Error> {
        match &mut self.kind {
            PortContextKind::Tcp(ctx) => ctx.prepare(config).await,
        }
    }

    pub async fn setup(&mut self, certs: &Keyring) -> Result<(), Error> {
        match &mut self.kind {
            PortContextKind::Tcp(ctx) => ctx.setup(certs).await,
        }
    }

    pub fn apply(&mut self, new: Self) {
        match (&mut self.kind, new.kind) {
            (PortContextKind::Tcp(old), PortContextKind::Tcp(new)) => old.apply(new),
        }
        self.entry = new.entry;
    }

    pub fn event(&mut self, event: PortContextEvent) {
        match &mut self.kind {
            PortContextKind::Tcp(ctx) => ctx.event(event),
        }
    }

    pub fn status(&self) -> &PortStatus {
        match &self.kind {
            PortContextKind::Tcp(ctx) => ctx.status(),
        }
    }
}

#[derive(Debug)]
pub enum PortContextKind {
    Tcp(TcpPortContext),
}
