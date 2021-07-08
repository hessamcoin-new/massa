use async_trait::async_trait;
use communication::network::network_controller::NetworkController;
use communication::protocol::protocol_controller::{
    NodeId, ProtocolController, ProtocolEvent, ProtocolEventType,
};
use communication::{
    network::establisher::{Connector, Establisher, Listener},
    CommunicationError,
};
use crypto::hash::Hash;
use models::block::Block;
use time::UTime;
use tokio::io::DuplexStream;
use tokio::sync::mpsc::{self, Receiver, Sender};

pub type ReadHalf = tokio::io::ReadHalf<DuplexStream>;
pub type WriteHalf = tokio::io::WriteHalf<DuplexStream>;

#[derive(Debug)]
pub struct BlankListener;

#[async_trait]
impl Listener<ReadHalf, WriteHalf> for BlankListener {
    async fn accept(&mut self) -> std::io::Result<(ReadHalf, WriteHalf, std::net::SocketAddr)> {
        unreachable!();
    }
}

#[derive(Debug)]
pub struct BlankConnector;

#[async_trait]
impl Connector<ReadHalf, WriteHalf> for BlankConnector {
    async fn connect(
        &mut self,
        _addr: std::net::SocketAddr,
    ) -> std::io::Result<(ReadHalf, WriteHalf)> {
        unreachable!();
    }
}

#[derive(Debug)]
pub struct BlankEstablisher;

#[async_trait]
impl Establisher for BlankEstablisher {
    type ReaderT = ReadHalf;
    type WriterT = WriteHalf;
    type ListenerT = BlankListener;
    type ConnectorT = BlankConnector;

    async fn get_listener(
        &mut self,
        _addr: std::net::SocketAddr,
    ) -> std::io::Result<Self::ListenerT> {
        unreachable!()
    }

    async fn get_connector(
        &mut self,
        _timeout_duration: UTime,
    ) -> std::io::Result<Self::ConnectorT> {
        unreachable!()
    }
}

#[derive(Debug)]
pub struct BlankNetworkController;

#[async_trait]
impl NetworkController for BlankNetworkController {
    type EstablisherT = BlankEstablisher;
    type ReaderT = ReadHalf;
    type WriterT = WriteHalf;

    async fn stop(mut self) -> Result<(), CommunicationError> {
        unreachable!();
    }

    async fn wait_event(
        &mut self,
    ) -> Result<
        communication::network::network_controller::NetworkEvent<Self::ReaderT, Self::WriterT>,
        CommunicationError,
    > {
        unreachable!();
    }

    async fn merge_advertised_peer_list(
        &mut self,
        _ips: Vec<std::net::IpAddr>,
    ) -> Result<(), CommunicationError> {
        unreachable!();
    }

    async fn get_advertisable_peer_list(
        &mut self,
    ) -> Result<Vec<std::net::IpAddr>, CommunicationError> {
        unreachable!();
    }

    async fn connection_closed(
        &mut self,
        _id: communication::network::network_controller::ConnectionId,
        _reason: communication::network::network_controller::ConnectionClosureReason,
    ) -> Result<(), CommunicationError> {
        unreachable!();
    }

    async fn connection_alive(
        &mut self,
        _id: communication::network::network_controller::ConnectionId,
    ) -> Result<(), CommunicationError> {
        unreachable!();
    }
}

#[derive(Clone, Debug)]
pub enum MockProtocolCommand {
    PropagateBlock { hash: Hash, block: Block },
}

pub fn new() -> (MockProtocolController, MockProtocolControllerInterface) {
    let (protocol_event_tx, protocol_event_rx) = mpsc::channel::<ProtocolEvent>(1024);
    let (protocol_command_tx, protocol_command_rx) = mpsc::channel::<MockProtocolCommand>(1024);
    (
        MockProtocolController {
            protocol_event_rx,
            protocol_command_tx,
        },
        MockProtocolControllerInterface {
            protocol_event_tx,
            protocol_command_rx,
        },
    )
}

#[derive(Debug)]
pub struct MockProtocolController {
    protocol_event_rx: Receiver<ProtocolEvent>,
    protocol_command_tx: Sender<MockProtocolCommand>,
}

#[async_trait]
impl ProtocolController for MockProtocolController {
    type NetworkControllerT = BlankNetworkController;

    async fn wait_event(
        &mut self,
    ) -> Result<communication::protocol::protocol_controller::ProtocolEvent, CommunicationError>
    {
        self.protocol_event_rx
            .recv()
            .await
            .ok_or(CommunicationError::GeneralProtocolError(format!(
                "failed retrieving protocol controller event in mock protocol controller"
            )))
    }

    async fn stop(mut self) -> Result<(), CommunicationError> {
        Ok(())
    }

    async fn propagate_block(
        &mut self,
        hash: Hash,
        block: &models::block::Block,
    ) -> Result<(), CommunicationError> {
        self.protocol_command_tx
            .send(MockProtocolCommand::PropagateBlock {
                hash,
                block: block.clone(),
            })
            .await
            .expect("could not send mock protocol command");
        Ok(())
    }
}

#[derive(Debug)]
pub struct MockProtocolControllerInterface {
    protocol_event_tx: Sender<ProtocolEvent>,
    protocol_command_rx: Receiver<MockProtocolCommand>,
}

impl MockProtocolControllerInterface {
    pub async fn wait_command(&mut self) -> Option<MockProtocolCommand> {
        Some(self.protocol_command_rx.recv().await?)
    }

    pub async fn receive_block(&mut self, source_node_id: NodeId, block: &Block) {
        self.protocol_event_tx
            .send(ProtocolEvent(
                source_node_id,
                ProtocolEventType::ReceivedBlock(block.clone()),
            ))
            .await
            .expect("could not send protocol event");
    }

    pub async fn receive_transaction(&mut self, source_node_id: NodeId, transaction: &String) {
        self.protocol_event_tx
            .send(ProtocolEvent(
                source_node_id,
                ProtocolEventType::ReceivedTransaction(transaction.clone()),
            ))
            .await
            .expect("could not send protocol event");
    }

    pub async fn receive_block_ask(&mut self, source_node_id: NodeId, hash: Hash) {
        self.protocol_event_tx
            .send(ProtocolEvent(
                source_node_id,
                ProtocolEventType::AskedBlock(hash),
            ))
            .await
            .expect("could not send protocol event");
    }
}
