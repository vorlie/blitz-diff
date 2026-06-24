use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModProfile {
    pub name: String,
    pub enabled_mod_ids: Vec<Uuid>,
    pub load_orders: Vec<(Uuid, u32)>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProfilesStore {
    pub profiles: Vec<ModProfile>,
}

impl ProfilesStore {
    pub fn get(&self, name: &str) -> Option<&ModProfile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut ModProfile> {
        self.profiles.iter_mut().find(|p| p.name == name)
    }

    pub fn upsert(&mut self, profile: ModProfile) {
        if let Some(existing) = self.get_mut(&profile.name) {
            *existing = profile;
        } else {
            self.profiles.push(profile);
        }
    }

    pub fn remove(&mut self, name: &str) {
        self.profiles.retain(|p| p.name != name);
    }
}
