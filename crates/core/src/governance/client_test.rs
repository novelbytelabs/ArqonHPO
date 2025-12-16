
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::governance::{GovernanceClient, MockBus, EnforcementAction, UpstreamMessage, DownstreamMessage};

    #[test]
    fn test_client_bus_interaction() {
        let (client, bus) = GovernanceClient::new_mock();
        
        // 1. Client sends telemetry
        let tele = UpstreamMessage::Telemetry {
            timestamp_us: 12345,
            node_id: "node_1".to_string(),
            metrics: HashMap::new(),
        };
        client.send(tele.clone()).unwrap();
        
        // Bus receives it
        let received = bus.recv().unwrap();
        match received {
            UpstreamMessage::Telemetry { timestamp_us, .. } => assert_eq!(timestamp_us, 12345),
            _ => panic!("Wrong message type"),
        }
        
        // 2. Bus sends enforcement
        let enforce = DownstreamMessage::Enforce {
            action: EnforcementAction::EmergencyStop,
            reason: "Safety breach".to_string(),
        };
        bus.send(enforce);
        
        // Client receives it
        let received_down = client.try_recv().unwrap();
        match received_down {
            DownstreamMessage::Enforce { reason, .. } => assert_eq!(reason, "Safety breach"),
            _ => panic!("Wrong message type"),
        }
    }
}
