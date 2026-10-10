//! Asset manifest: every asset is referenced by id; paths live in `assets/manifest.json`.
//! To replace an asset, overwrite the file or change its path in the manifest.
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct AssetEntry {
    pub path: String,
    pub kind: String,
    #[serde(default)]
    pub use_: String,
}

#[derive(Resource, Default, Debug)]
pub struct AssetManifest(pub HashMap<String, AssetEntry>);

impl AssetManifest {
    pub fn load() -> Self {
        let text = std::fs::read_to_string("assets/manifest.json").unwrap_or_else(|e| {
            warn!("assets/manifest.json missing ({e}); run `sh tools/gen_all.sh`");
            "{}".into()
        });
        // manifest uses key "use" (reserved word in Rust) -> remap
        let raw: HashMap<String, serde_json::Value> = serde_json::from_str(&text).unwrap_or_default();
        let map = raw
            .into_iter()
            .filter_map(|(id, v)| {
                Some((id, AssetEntry {
                    path: v.get("path")?.as_str()?.to_string(),
                    kind: v.get("kind")?.as_str()?.to_string(),
                    use_: v.get("use").and_then(|u| u.as_str()).unwrap_or("").to_string(),
                }))
            })
            .collect();
        Self(map)
    }

    /// Path relative to `assets/` (what AssetServer wants). Panics-free: logs and returns a fallback.
    pub fn path(&self, id: &str) -> String {
        match self.0.get(id) {
            Some(e) => e.path.clone(),
            None => {
                error!("unknown asset id '{id}' (see assets.md)");
                "missing.png".to_string()
            }
        }
    }

    pub fn load_image(&self, id: &str, server: &AssetServer) -> Handle<Image> {
        server.load(self.path(id))
    }
}

pub struct ManifestPlugin;
impl Plugin for ManifestPlugin {
    fn build(&self, app: &mut App) {
        let m = AssetManifest::load();
        info!("asset manifest: {} entries", m.0.len());
        app.insert_resource(m);
    }
}
