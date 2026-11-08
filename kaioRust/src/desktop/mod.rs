pub mod app;
pub mod daemon;
pub mod embedded;
pub mod gui;
pub mod ipc;
pub mod model;
pub mod protocol;
pub mod theme;
pub mod view;

pub use app::KaiokenDesktopController;
pub use daemon::DaemonService;
pub use embedded::EmbeddedRuntime;
pub use ipc::{default_ipc_endpoint, DaemonIpcClient, DaemonIpcServer};
pub use model::{
    ActiveWorkspaceTab, AgentSessionModel, FailureCardModel, MemoryGraphModel,
    ResearchDossierModel, ReviewStateModel, SettingsModel, WikiChapterModel,
};
pub use protocol::{IpcEvent, IpcRequest, IpcResponse, PolicyTier, ToolProposal};
pub use theme::DesktopTheme;
pub use view::KaiokenAppWindow;
