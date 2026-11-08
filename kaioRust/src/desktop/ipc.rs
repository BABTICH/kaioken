use crate::desktop::protocol::{IpcEvent, IpcRequest, IpcResponse, WireMessage};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};

pub const DEFAULT_TCP_PORT: u16 = 47890;
/// Upper bound on a daemon round-trip. Without it a dead peer leaves the UI
/// waiting forever on a request that will never be answered.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Default endpoint.
///
/// Loopback TCP is used on every platform. Named pipes and Unix sockets are
/// not implemented yet; advertising them in `default_ipc_endpoint()` made the
/// desktop try an address no server binds. TCP loopback works identically on
/// Windows, macOS, Linux, and inside WSL2.
pub fn default_ipc_endpoint() -> String {
    format!("127.0.0.1:{}", DEFAULT_TCP_PORT)
}

/// An IPC client that communicates with the `kaioken watch` background daemon.
pub struct DaemonIpcClient {
    endpoint: String,
    next_id: AtomicU64,
    outgoing_tx: mpsc::Sender<WireMessage>,
    pending_requests: Arc<Mutex<HashMap<u64, oneshot::Sender<IpcResponse>>>>,
    event_tx: broadcast::Sender<IpcEvent>,
    is_connected: Arc<AtomicBool>,
}

impl DaemonIpcClient {
    /// Create a simulated in-memory IPC client for testing or standalone execution.
    pub fn new_mock() -> (
        Self,
        mpsc::Receiver<WireMessage>,
        broadcast::Sender<IpcEvent>,
    ) {
        let (outgoing_tx, outgoing_rx) = mpsc::channel(128);
        let (event_tx, _) = broadcast::channel(128);
        let client = Self {
            endpoint: "mock://in-memory".to_string(),
            next_id: AtomicU64::new(1),
            outgoing_tx,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            event_tx: event_tx.clone(),
            is_connected: Arc::new(AtomicBool::new(true)),
        };
        (client, outgoing_rx, event_tx)
    }

    /// Connect with a bounded wait, so an unreachable endpoint cannot stall a
    /// CLI startup for the OS connect timeout.
    pub async fn connect_with_timeout(addr: &str, timeout: Duration) -> Result<Arc<Self>> {
        match tokio::time::timeout(timeout, Self::connect_tcp(addr)).await {
            Ok(result) => result,
            Err(_) => Err(anyhow!(
                "connection to {} timed out after {}ms",
                addr,
                timeout.as_millis()
            )),
        }
    }

    /// Connect to the default local daemon endpoint.
    pub async fn connect_local() -> Result<Arc<Self>> {
        Self::connect_with_timeout(&default_ipc_endpoint(), Duration::from_millis(1500)).await
    }

    /// Connect to a TCP-based daemon endpoint (e.g. "127.0.0.1:47890").
    pub async fn connect_tcp(addr: &str) -> Result<Arc<Self>> {
        let stream = TcpStream::connect(addr).await?;
        let (reader, mut writer) = stream.into_split();
        let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<WireMessage>(128);
        let (event_tx, _) = broadcast::channel::<IpcEvent>(128);
        let pending = Arc::new(Mutex::new(
            HashMap::<u64, oneshot::Sender<IpcResponse>>::new(),
        ));
        let is_connected = Arc::new(AtomicBool::new(true));

        // Outgoing write loop
        let is_conn_writer = is_connected.clone();
        tokio::spawn(async move {
            while let Some(msg) = outgoing_rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if writer.write_all(json.as_bytes()).await.is_err()
                        || writer.write_all(b"\n").await.is_err()
                        || writer.flush().await.is_err()
                    {
                        is_conn_writer.store(false, Ordering::SeqCst);
                        break;
                    }
                }
            }
        });

        // Incoming read loop
        let pending_reader = pending.clone();
        let event_tx_reader = event_tx.clone();
        let is_conn_reader = is_connected.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(reader).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(msg) = serde_json::from_str::<WireMessage>(&line) {
                    match msg {
                        WireMessage::Response { id, response } => {
                            if let Some(sender) = pending_reader.lock().await.remove(&id) {
                                let _ = sender.send(response);
                            }
                        }
                        WireMessage::Event { event } => {
                            let _ = event_tx_reader.send(event);
                        }
                        _ => {}
                    }
                }
            }
            is_conn_reader.store(false, Ordering::SeqCst);
        });

        Ok(Arc::new(Self {
            endpoint: addr.to_string(),
            next_id: AtomicU64::new(1),
            outgoing_tx,
            pending_requests: pending,
            event_tx,
            is_connected,
        }))
    }

    /// Send a request to the daemon and await its response.
    pub async fn send_request(&self, request: IpcRequest) -> Result<IpcResponse> {
        self.send_request_with_timeout(request, DEFAULT_REQUEST_TIMEOUT)
            .await
    }

    /// Send a request with an explicit response deadline.
    ///
    /// A timed-out request is removed from the pending map, so a dead peer
    /// cannot leak one entry (and one oneshot sender) per call.
    pub async fn send_request_with_timeout(
        &self,
        request: IpcRequest,
        timeout: Duration,
    ) -> Result<IpcResponse> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.pending_requests.lock().await.insert(id, tx);

        let wire = WireMessage::Request { id, request };
        if let Err(error) = self.outgoing_tx.send(wire).await {
            self.pending_requests.lock().await.remove(&id);
            return Err(anyhow!("Failed to send IPC request: {}", error));
        }

        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(error)) => Err(anyhow!("IPC response dropped: {}", error)),
            Err(_) => {
                self.pending_requests.lock().await.remove(&id);
                Err(anyhow!(
                    "IPC request timed out after {}ms",
                    timeout.as_millis()
                ))
            }
        }
    }

    /// Subscribe to real-time events streamed from the daemon.
    pub fn subscribe_events(&self) -> broadcast::Receiver<IpcEvent> {
        self.event_tx.subscribe()
    }

    /// Broadcast an incoming event (used in mock or embedded mode).
    pub fn emit_event(&self, event: IpcEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Manually resolve a pending request (used in mock mode).
    pub async fn mock_respond(&self, id: u64, response: IpcResponse) -> bool {
        if let Some(tx) = self.pending_requests.lock().await.remove(&id) {
            tx.send(response).is_ok()
        } else {
            false
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub async fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst)
    }

    /// Synchronous read for UI loops that cannot await.
    pub fn is_connected_now(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst)
    }
}

/// Standalone or embedded daemon IPC server (Section 2.1).
pub struct DaemonIpcServer {
    listener: TcpListener,
    port: u16,
    event_tx: broadcast::Sender<IpcEvent>,
}

impl DaemonIpcServer {
    /// Bind to a local port or ephemeral port.
    pub async fn bind(port: u16) -> Result<Self> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        let bound_port = listener.local_addr()?.port();
        let (event_tx, _) = broadcast::channel(128);
        Ok(Self {
            listener,
            port: bound_port,
            event_tx,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn addr(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    /// Sender used to push `IpcEvent::FileChanged` and friends to every
    /// connected client. Cloning it is how the watcher publishes updates
    /// without knowing which clients are attached.
    pub fn event_sender(&self) -> broadcast::Sender<IpcEvent> {
        self.event_tx.clone()
    }

    /// Accept client connections, answer requests, and stream broadcast events.
    pub async fn serve<F, Fut>(self, handler: F) -> Result<()>
    where
        F: Fn(IpcRequest) -> Fut + Send + Sync + 'static + Clone,
        Fut: std::future::Future<Output = IpcResponse> + Send + 'static,
    {
        let event_tx = self.event_tx.clone();
        while let Ok((stream, _)) = self.listener.accept().await {
            let handler = handler.clone();
            let mut event_rx = event_tx.subscribe();
            tokio::spawn(async move {
                let (reader, mut writer) = stream.into_split();
                let mut lines = BufReader::new(reader).lines();

                loop {
                    tokio::select! {
                        line = lines.next_line() => {
                            let Ok(Some(line)) = line else { break };
                            if let Ok(WireMessage::Request { id, request }) =
                                serde_json::from_str::<WireMessage>(&line)
                            {
                                let response = handler(request).await;
                                let reply = WireMessage::Response { id, response };
                                if !write_wire(&mut writer, &reply).await {
                                    break;
                                }
                            }
                        }
                        event = event_rx.recv() => {
                            match event {
                                Ok(event) => {
                                    if !write_wire(&mut writer, &WireMessage::Event { event }).await {
                                        break;
                                    }
                                }
                                // A slow client missed intermediate events; keep
                                // the connection and continue from the newest one.
                                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                                Err(broadcast::error::RecvError::Closed) => break,
                            }
                        }
                    }
                }
            });
        }
        Ok(())
    }
}

async fn write_wire(writer: &mut OwnedWriteHalf, message: &WireMessage) -> bool {
    let Ok(json) = serde_json::to_string(message) else {
        return false;
    };
    writer.write_all(json.as_bytes()).await.is_ok()
        && writer.write_all(b"\n").await.is_ok()
        && writer.flush().await.is_ok()
}
