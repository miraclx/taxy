use crate::{certs::Cert, server::rpc::ErasedRpcMethod};
use std::sync::Arc;

pub enum ServerCommand {
    AddCert {
        cert: Arc<Cert>,
    },
    SetBroadcastEvents {
        enabled: bool,
    },
    StopHttpChallenges,
    CallMethod {
        id: usize,
        arg: Box<dyn ErasedRpcMethod>,
    },
}

impl std::fmt::Debug for ServerCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddCert { cert } => f.debug_struct("AddCert").field("id", &cert.id()).finish(),
            Self::SetBroadcastEvents { enabled } => f
                .debug_struct("SetBroadcastEvents")
                .field("enabled", enabled)
                .finish(),
            Self::StopHttpChallenges => f.debug_struct("StopHttpChallenges").finish(),
            Self::CallMethod { id, .. } => f.debug_struct("CallMethod").field("id", id).finish(),
        }
    }
}
