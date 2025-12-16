//! Mock Client for Governance integration
//!
//! Simulates the ArqonBus connection. In a real system, this would use NATS/gRPC.
//! Here we use channels to test the interaction model.

use std::sync::mpsc::{Sender, Receiver, channel};
use std::collections::HashMap;
use super::{UpstreamMessage, DownstreamMessage};

/// Client for interacting with the Governance Control Plane
pub struct GovernanceClient {
    tx: Sender<UpstreamMessage>,
    rx: Receiver<DownstreamMessage>,
}

impl GovernanceClient {
    /// Create a pair: (Client, MockBus)
    pub fn new_mock() -> (Self, MockBus) {
        let (tx_up, rx_up) = channel();
        let (tx_down, rx_down) = channel();
        
        let client = Self {
            tx: tx_up,
            rx: rx_down,
        };
        
        let bus = MockBus {
            tx: tx_down,
            rx: rx_up,
        };
        
        (client, bus)
    }
    
    /// Send telemetry or proposal upstream
    pub fn send(&self, msg: UpstreamMessage) -> Result<(), String> {
        self.tx.send(msg).map_err(|e| e.to_string())
    }
    
    /// Check for downstream instructions (non-blocking)
    pub fn try_recv(&self) -> Option<DownstreamMessage> {
        self.rx.try_recv().ok()
    }
}

/// Mock ArqonBus for testing
pub struct MockBus {
    tx: Sender<DownstreamMessage>,
    rx: Receiver<UpstreamMessage>,
}

impl MockBus {
    /// Send instruction downstream
    pub fn send(&self, msg: DownstreamMessage) {
        let _ = self.tx.send(msg);
    }
    
    /// Receive upstream message
    pub fn recv(&self) -> Option<UpstreamMessage> {
        self.rx.try_recv().ok()
    }
}

#[cfg(test)]
#[path = "client_test.rs"]
mod client_test;
