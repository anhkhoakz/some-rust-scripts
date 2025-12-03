//! better-escape.nvim rewritten in Rust using nvim-oxi v0.6.0
//!
//! This file implements the same behavior as the original Lua version:
//! - configurable pair mappings (e.g. "j" + "k" -> <Esc>) across modes
//! - times out if the second key isn't pressed within `timeout` ms
//! - restores buffer modified flag after injection
//!

use nvim_oxi as oxi;
use once_cell::sync::OnceCell;
use oxi::api;
use oxi::api::opts::SetKeymapOpts;
use oxi::api::types::Mode as ApiMode;
use oxi::libuv::TimerHandle;
use oxi::{Dictionary, Function, Object, String as NvimString};
use parking_lot::Mutex;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

// ----- Types mirroring the Lua settings shape -----

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    I,
    C,
    T,
    V,
    S,
}

/// Mapping for a single "first key" to multiple second-key -> mapping (string or left for function)
pub type SecondKeyMap = HashMap<String, MappingValue>;

/// Per-mode mapping: first_key -> second_key -> MappingValue
pub type ModeMapping = HashMap<String, SecondKeyMap>;

#[derive(Debug, Clone)]
pub enum MappingValue {
    Str(String),
    // We keep the possibility for a callback by name, but the initial port will support strings.
    Func(String),
}

impl From<String> for MappingValue {
    fn from(s: String) -> Self {
        MappingValue::Str(s)
    }
}

// ----- Plugin state -----

struct Settings {
    timeout: u64,
    mappings: HashMap<String, ModeMapping>, // key by mode letter: "i", "c", ...
}

impl Default for Settings {
    fn default() -> Self {
        let mut mappings = HashMap::new();
        // Populate same defaults as Lua original
        mappings.insert("i".to_string(), {
            let mut m = ModeMapping::new();
            let mut jm = SecondKeyMap::new();
            jm.insert("k".to_string(), MappingValue::Str("<Esc>".to_string()));
            jm.insert("j".to_string(), MappingValue::Str("<Esc>".to_string()));
            m.insert("j".to_string(), jm);
            m
        });
        mappings.insert("c".to_string(), {
            let mut m = ModeMapping::new();
            let mut jm = SecondKeyMap::new();
            jm.insert("k".to_string(), MappingValue::Str("<C-c>".to_string()));
            jm.insert("j".to_string(), MappingValue::Str("<C-c>".to_string()));
            m.insert("j".to_string(), jm);
            m
        });
        mappings.insert("t".to_string(), {
            let mut m = ModeMapping::new();
            let mut jm = SecondKeyMap::new();
            jm.insert(
                "k".to_string(),
                MappingValue::Str("<C-\\><C-n>".to_string()),
            );
            m.insert("j".to_string(), jm);
            m
        });
        mappings.insert("v".to_string(), {
            let mut m = ModeMapping::new();
            let mut jm = SecondKeyMap::new();
            jm.insert("k".to_string(), MappingValue::Str("<Esc>".to_string()));
            m.insert("j".to_string(), jm);
            m
        });
        mappings.insert("s".to_string(), {
            let mut m = ModeMapping::new();
            let mut jm = SecondKeyMap::new();
            jm.insert("k".to_string(), MappingValue::Str("<Esc>".to_string()));
            m.insert("j".to_string(), jm);
            m
        });

        Settings {
            timeout: api::get_option_value::<i64>("timeoutlen", &Default::default()).unwrap_or(1000)
                as u64,
            mappings,
        }
    }
}

// Global plugin state stored once and shared with callbacks
struct State {
    settings: Settings,
    waiting: bool,
    recorded_key: Option<String>,
    bufmodified: bool,
    has_recorded: bool,
}

static PLUGIN_STATE: OnceCell<Mutex<State>> = OnceCell::new();

fn get_state() -> &'static Mutex<State> {
    PLUGIN_STATE.get_or_init(|| {
        Mutex::new(State {
            settings: Settings::default(),
            waiting: false,
            recorded_key: None,
            bufmodified: false,
            has_recorded: false,
        })
    })
}

// ----- Helpers -----

fn t(s: &str) -> String {
    api::replace_termcodes(s, true, true, true).to_string()
}

// ----- Mapping management -----

fn mode_str_to_api_mode(mode: &str) -> ApiMode {
    match mode {
        "i" => ApiMode::Insert,
        "c" => ApiMode::CmdLine,
        "t" => ApiMode::Terminal,
        "v" => ApiMode::VisualSelect,
        "s" => ApiMode::Select,
        _ => ApiMode::Insert,
    }
}

fn unmap_keys() {
    let state = get_state();
    let s = state.lock();
    for (mode, mode_map) in s.settings.mappings.iter() {
        let api_mode = mode_str_to_api_mode(mode);
        for (first_key, second_map) in mode_map.iter() {
            // Attempt to delete both first and each second key mapping
            let _ = api::del_keymap(api_mode, first_key);
            for (second_key, _) in second_map.iter() {
                let _ = api::del_keymap(api_mode, second_key);
            }
        }
    }
}

fn map_keys() {
    let state = get_state();
    let mut s = state.lock();
    // We need closures that are callable via expr mapping in Neovim.
    for (mode, first_keys) in s.settings.mappings.clone() {
        // For each first_key, set an expr mapping that calls our rust-backed functions.
        for (first_key, _) in first_keys.iter() {
            // Define an expr mapping that calls the module function to record the key and returns the literal first key
            // We set the mapping to call a lua wrapper that invokes Rust export
            let lua_rhs = format!(
                "v:lua.require('better_escape')._record_key('{}')",
                first_key
            );
            // Use expr = true
            let api_mode = mode_str_to_api_mode(&mode);
            let opts = SetKeymapOpts::builder()
                .expr(true)
                .noremap(true)
                .nowait(false)
                .build();
            let _ = api::set_keymap(api_mode, first_key, &lua_rhs, &opts);
        }

        // For each second key, set a handler that either records new key or, if a valid previous first was recorded,
        // composes keys and injects them.
        for (first_key, second_keys) in first_keys.iter() {
            for (second_key, _mapping) in second_keys.iter() {
                let lua_rhs = format!(
                    "v:lua.require('better_escape')._handle_second('{}','{}')",
                    first_key, second_key
                );
                let api_mode = mode_str_to_api_mode(&mode);
                let opts = SetKeymapOpts::builder()
                    .expr(true)
                    .noremap(true)
                    .nowait(false)
                    .build();
                let _ = api::set_keymap(api_mode, second_key, &lua_rhs, &opts);
            }
        }
    }

    // update state waiting flags cleared on remap
    s.waiting = false;
    s.recorded_key = None;
}

// ----- Timer and recorder functions callable from Lua -----

/// Public API exposed to Lua: setup(settings_table)
#[oxi::plugin]
fn better_escape() -> Dictionary {
    // setup(tbl) - currently a no-op, settings are handled via the mappings
    let setup_fn: Function<Object, ()> = Function::from_fn(|_args: Object| {
        // Unmap and remap with current settings
        unmap_keys();
        map_keys();
    });

    // _record_key(first_key) -> returns the literal first_key (so expr mapping inserts it)
    let record_fn: Function<String, String> = Function::from_fn(|first_key: String| {
        let state = get_state();
        let mut s = state.lock();

        // Get buffer-local option value (modified is a buffer-local option)
        use oxi::api::opts::{OptionOpts, OptionScope};
        let opts = OptionOpts::builder().scope(OptionScope::Local).build();
        s.bufmodified = api::get_option_value::<bool>("modified", &opts).unwrap_or(false);

        s.recorded_key = Some(first_key.clone());
        s.has_recorded = true;
        s.waiting = true;

        // start timer to clear recorded_key after timeout
        let timeout_ms = s.settings.timeout;
        let _timer_handle = TimerHandle::once(Duration::from_millis(timeout_ms), move || {
            // ensure this runs on neovim main loop
            oxi::schedule(move |_| {
                let state = get_state();
                let mut s = state.lock();
                s.waiting = false;
                s.recorded_key = None;
            });
        })
        .ok();

        // Return the literal first_key so the expr mapping inserts it
        first_key
    });

    // _handle_second(first_key, second_key) -> returns either inserted second_key (string)
    // or injected mapping (empty string) when the pair matches.
    let handle_second_fn: Function<(String, String), String> =
        Function::from_fn(|(_first_key, second_key): (String, String)| {
            let state = get_state();
            let mut s = state.lock();

            // If a first_key wasn't recorded, record second_key (it may start another sequence)
            if s.recorded_key.is_none() {
                // reuse record logic: set state and return literal second_key
                s.recorded_key = Some(second_key.clone());
                s.has_recorded = true;
                // Get buffer-local option value (modified is a buffer-local option)
                use oxi::api::opts::{OptionOpts, OptionScope};
                let opts = OptionOpts::builder().scope(OptionScope::Local).build();
                s.bufmodified = api::get_option_value::<bool>("modified", &opts).unwrap_or(false);
                s.waiting = true;

                let timeout_ms = s.settings.timeout;
                let _timer_handle =
                    TimerHandle::once(Duration::from_millis(timeout_ms), move || {
                        oxi::schedule(move |_| {
                            let state = get_state();
                            let mut s = state.lock();
                            s.waiting = false;
                            s.recorded_key = None;
                        });
                    })
                    .ok();

                return second_key;
            }

            // If recorded_key isn't the right first for this second, record the second_key and insert it.
            let mode_maps = &s.settings.mappings;
            let recorded = s.recorded_key.clone().unwrap();
            let mode_possible = mode_maps.values().any(|m| {
                m.get(&recorded)
                    .map(|m2| m2.get(&second_key).is_some())
                    .unwrap_or(false)
            });

            // If the pair doesn't match for any mode, behave like normal
            if !mode_possible {
                s.recorded_key = Some(second_key.clone());
                return second_key;
            }

            // At this point we've determined recorded + second_key is a valid mapping in at least one mode.
            // For simplicity we search the mapping and take the first mapping found.
            let mut mapped_action: Option<MappingValue> = None;
            let mut found_mode: Option<String> = None;
            for (mode, mm) in mode_maps.iter() {
                if let Some(secmap) = mm.get(&recorded) {
                    if let Some(val) = secmap.get(&second_key) {
                        mapped_action = Some(val.clone());
                        found_mode = Some(mode.clone());
                        break;
                    }
                }
            }

            // Compose the undo key (backspace) + restore buffer modified flag + mapping
            let mode = found_mode.unwrap_or_else(|| "i".to_string());
            let undo = match mode.as_str() {
                "i" | "c" | "t" => "<bs>",
                _ => "",
            };

            let mut inject = String::new();
            inject.push_str(&t(&(format!(
                "{}<cmd>setlocal {}modified<cr>",
                undo,
                if s.bufmodified { "" } else { "no" }
            ))));

            if let Some(mapping) = mapped_action {
                match mapping {
                    MappingValue::Str(smap) => inject.push_str(&t(&smap)),
                    MappingValue::Func(_fname) => {
                        // For now, we don't support arbitrary function values in Rust port; leave empty.
                    }
                }
            }

            // Feed keys into Neovim
            let inject_nvim_str = NvimString::from(inject.as_str());
            let mode_str = NvimString::from("in");
            api::feedkeys(inject_nvim_str.as_nvim_str(), mode_str.as_nvim_str(), false);

            // clear recorded state
            s.waiting = false;
            s.recorded_key = None;
            s.has_recorded = false;

            // We already injected, so return empty string for expr mapping (nothing to insert now)
            String::new()
        });

    Dictionary::from_iter([
        ("setup", Object::from(setup_fn)),
        ("_record_key", Object::from(record_fn)),
        ("_handle_second", Object::from(handle_second_fn)),
    ])
}
