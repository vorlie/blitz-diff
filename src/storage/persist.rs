use crate::model::{
    AppConfig, ClientState, LegacyHashesDB, ModCatalog, ProfilesStore,
};

const APP_NAME: &str = "blitz-diff";

pub fn load_config() -> AppConfig {
    confy::load(APP_NAME, "config").unwrap_or_default()
}

pub fn save_config(config: &AppConfig) {
    let _ = confy::store(APP_NAME, "config", config);
}

pub fn load_catalog() -> ModCatalog {
    confy::load(APP_NAME, "catalog").unwrap_or_default()
}

pub fn save_catalog(catalog: &ModCatalog) {
    let _ = confy::store(APP_NAME, "catalog", catalog);
}

pub fn load_profiles() -> ProfilesStore {
    confy::load(APP_NAME, "profiles").unwrap_or_default()
}

pub fn save_profiles(profiles: &ProfilesStore) {
    let _ = confy::store(APP_NAME, "profiles", profiles);
}

pub fn load_client_state(client: crate::model::ClientId) -> ClientState {
    let name = state_file_name(client);
    confy::load(APP_NAME, name.as_str()).unwrap_or_default()
}

pub fn save_client_state(client: crate::model::ClientId, state: &ClientState) {
    let name = state_file_name(client);
    let _ = confy::store(APP_NAME, name.as_str(), state);
}

fn state_file_name(client: crate::model::ClientId) -> String {
    match client {
        crate::model::ClientId::Steam => "state_steam".to_string(),
        crate::model::ClientId::Wgc => "state_wgc".to_string(),
    }
}

pub fn migrate_legacy_hashes() -> Option<(ClientState, ClientState)> {
    let legacy: Result<LegacyHashesDB, _> = confy::load(APP_NAME, "hashes_db");
    let legacy = legacy.ok()?;
    if legacy.tracked_hashes.is_empty() {
        return None;
    }

    let steam = ClientState::from_legacy(&legacy, "legacy-migration");
    let wgc = ClientState::from_legacy(&legacy, "legacy-migration");
    Some((steam, wgc))
}

pub struct AppStorage {
    pub config: AppConfig,
    pub catalog: ModCatalog,
    pub profiles: ProfilesStore,
    pub steam_state: ClientState,
    pub wgc_state: ClientState,
    pub migration_note: Option<String>,
}

impl AppStorage {
    pub fn load() -> Self {
        let config = load_config();
        let catalog = load_catalog();
        let profiles = load_profiles();
        let mut steam_state = load_client_state(crate::model::ClientId::Steam);
        let mut wgc_state = load_client_state(crate::model::ClientId::Wgc);
        let mut migration_note = None;

        if steam_state.files.is_empty() && wgc_state.files.is_empty() {
            if let Some((steam, wgc)) = migrate_legacy_hashes() {
                steam_state = steam;
                wgc_state = wgc;
                save_client_state(crate::model::ClientId::Steam, &steam_state);
                save_client_state(crate::model::ClientId::Wgc, &wgc_state);
                migration_note = Some(
                    "Migrated legacy hashes_db.toml to per-client state files.".to_string(),
                );
            }
        }

        Self {
            config,
            catalog,
            profiles,
            steam_state,
            wgc_state,
            migration_note,
        }
    }

    pub fn client_state_mut(&mut self, client: crate::model::ClientId) -> &mut ClientState {
        match client {
            crate::model::ClientId::Steam => &mut self.steam_state,
            crate::model::ClientId::Wgc => &mut self.wgc_state,
        }
    }

    pub fn client_state(&self, client: crate::model::ClientId) -> &ClientState {
        match client {
            crate::model::ClientId::Steam => &self.steam_state,
            crate::model::ClientId::Wgc => &self.wgc_state,
        }
    }

    pub fn save_client(&self, client: crate::model::ClientId) {
        save_client_state(client, self.client_state(client));
    }
}
