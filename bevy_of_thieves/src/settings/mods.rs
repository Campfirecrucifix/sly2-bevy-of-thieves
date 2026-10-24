//! WIRE-OPT mod manager basics: `mods/<folder>/mod.toml` metadata, the default load order and override
//! conflicts. mod.toml (all keys optional; same flat TOML subset as settings.toml):
//! ```toml
//! name = "Magenta level"
//! version = "1.0"
//! author = "me"
//! load_order = 10        # default position among mods NOT listed in load_order.txt / settings.toml
//! description = "..."
//! ```
//! Order (first = lowest priority, last wins): settings.toml `[mods] order` if non-empty, else the
//! names in `mods/load_order.txt` (`# off: name` = disabled), then the remaining folders sorted by
//! (`load_order`, folder name). A conflict = the same override path (texture / sound / level / ISO file) or
//! the same text id provided by two or more enabled mods; the last one in the order wins and it is logged.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModInfo {
    /// folder name under mods/ (the id used by load_order.txt and settings.toml)
    pub folder: String,
    pub dir: PathBuf,
    pub name: String,
    pub version: String,
    pub author: String,
    pub load_order: i32,
    pub description: String,
}

/// Parse a mod.toml (unknown keys ignored, bad lines skipped).
pub fn parse_mod_toml(folder: &str, dir: &Path, text: &str) -> ModInfo {
    let mut m = ModInfo { folder: folder.into(), dir: dir.into(), name: folder.into(), ..Default::default() };
    for l in text.lines() {
        let l = l.trim();
        if l.starts_with('#') || l.starts_with('[') {
            continue;
        }
        let Some((k, v)) = l.split_once('=') else { continue };
        let v = v.trim();
        let s = v.strip_prefix('"').and_then(|r| r.rfind('"').map(|i| r[..i].replace("\\\"", "\"").replace("\\n", "\n")));
        match (k.trim(), s) {
            ("name", Some(s)) => m.name = s,
            ("version", Some(s)) => m.version = s,
            ("author", Some(s)) => m.author = s,
            ("description", Some(s)) => m.description = s,
            ("load_order", _) => m.load_order = v.split('#').next().unwrap_or("").trim().parse().unwrap_or(0),
            _ => {}
        }
    }
    m
}

/// Every folder under `dir` with its mod.toml (folders without one get defaults, name = folder).
pub fn scan(dir: &Path) -> Vec<ModInfo> {
    let Ok(rd) = std::fs::read_dir(dir) else { return vec![] };
    let mut v: Vec<ModInfo> = rd
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| {
            let folder = e.file_name().to_string_lossy().into_owned();
            let text = std::fs::read_to_string(e.path().join("mod.toml")).unwrap_or_default();
            parse_mod_toml(&folder, &e.path(), &text)
        })
        .collect();
    v.sort_by(|a, b| a.folder.cmp(&b.folder));
    v
}

/// Enabled mods (folder names) in load order; see the module doc.
pub fn order(infos: &[ModInfo], settings_order: &[String], load_order_txt: &str) -> Vec<String> {
    let known = |n: &str| infos.iter().any(|m| m.folder == n);
    if !settings_order.is_empty() {
        return settings_order.iter().filter(|n| known(n)).cloned().collect();
    }
    let off: Vec<&str> = load_order_txt.lines().filter_map(|l| l.trim().strip_prefix("# off:").map(str::trim)).collect();
    let mut out: Vec<String> = vec![];
    for l in load_order_txt.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        if known(l) && !out.iter().any(|o| o == l) {
            out.push(l.to_string());
        }
    }
    let mut rest: Vec<&ModInfo> = infos.iter().filter(|m| !out.contains(&m.folder) && !off.contains(&m.folder.as_str())).collect();
    rest.sort_by(|a, b| (a.load_order, &a.folder).cmp(&(b.load_order, &b.folder)));
    out.extend(rest.into_iter().map(|m| m.folder.clone()));
    out
}

/// One overridden resource provided by several mods.
#[derive(Clone, Debug, PartialEq)]
pub struct Conflict {
    /// relative path inside the mod ("textures/350144/1628.png") or "text id 0x1234"
    pub what: String,
    /// providers in load order; the last one wins
    pub mods: Vec<String>,
}

impl Conflict {
    pub fn winner(&self) -> &str {
        self.mods.last().map_or("", |s| s.as_str())
    }
}

fn walk(root: &Path, d: &Path, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(d) else { return };
    for e in rd.filter_map(|e| e.ok()) {
        let p = e.path();
        if p.is_dir() {
            walk(root, &p, out);
        } else if let Ok(r) = p.strip_prefix(root) {
            let r = r.to_string_lossy().replace('\\', "/");
            // metadata and text files are not overrides by path (text ids are compared below)
            if r != "mod.toml" && !r.starts_with("text/") {
                out.push(r.to_ascii_lowercase());
            }
        }
    }
}

/// Conflicts among `mods` (mod dirs in load order).
pub fn conflicts(mods: &[PathBuf]) -> Vec<Conflict> {
    let mut by: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for m in mods {
        let name = m.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let mut files = vec![];
        walk(m, m, &mut files);
        let mut ids: Vec<i32> = vec![];
        if let Ok(rd) = std::fs::read_dir(m.join("text")) {
            for f in rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "toml")) {
                ids.extend(crate::engine::disc::mods::parse_text_toml(&std::fs::read_to_string(&f).unwrap_or_default()).into_iter().map(|(i, _)| i));
            }
        }
        ids.sort();
        ids.dedup();
        for k in files.into_iter().chain(ids.into_iter().map(|i| format!("text id {i:#x}"))) {
            by.entry(k).or_default().push(name.clone());
        }
    }
    by.into_iter().filter(|(_, v)| v.len() > 1).map(|(what, mods)| Conflict { what, mods }).collect()
}

/// Log every conflict (`mods: conflict ... -> <winner> wins`); returns them for the menu.
pub fn log_conflicts(mods: &[PathBuf]) -> Vec<Conflict> {
    let c = conflicts(mods);
    for x in &c {
        log::warn!("mods: conflict {}: {} -> {} wins (last in load order)", x.what, x.mods.join(", "), x.winner());
    }
    c
}
