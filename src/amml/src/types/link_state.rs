use std::sync::{Arc, OnceLock};

use amm_client::link::ManagerLink;
use minimap_link::services::KeyFeed;
use minimap_runtime::services::Runtime;

// where the minimap stands with the mod manager
pub enum LinkState {
    // the link waits for the manager's tables; `slot` takes the runtime the frame handler of the link draws, `keys`
    // is the key feed that frame handler drains
    Waiting { link: ManagerLink, slot: Arc<OnceLock<Arc<Runtime>>>, keys: Arc<KeyFeed> },
    // the session runs
    Attached,
    // the manager is missing, gone or refused the mod; the minimap stays off until the next enable
    Off,
}
