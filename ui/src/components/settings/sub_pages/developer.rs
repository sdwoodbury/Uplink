use std::rc::Weak;

use dioxus::prelude::*;
use dioxus_desktop::use_window;
use kit::{
    elements::{button::Button, switch::Switch, Appearance},
    icons::Icon,
};
use shared::language::get_local_text;
use warp::logging::tracing::log;

use crate::{
    components::{
        debug_logger::{DebugLogger, DebugLoggerProps},
        settings::SettingSection,
    },
    config::Configuration,
    logger,
    state::{Action, State},
    window_manager::{WindowManagerCmd, WindowManagerCmdTx},
    WINDOW_CMD_CH,
};

#[allow(non_snake_case)]
pub fn DeveloperSettings(cx: Scope) -> Element {
    log::debug!("Developer settings page rendered.");
    let state = use_shared_state::<State>(cx)?;
    let mut config = Configuration::load_or_default();
    let window = use_window(cx);

    cx.render(rsx!(
        div {
            id: "settings-developer",
            aria_label: "settings-developer",
            SettingSection {
                section_label: get_local_text("settings-developer.developer-mode"),
                section_description: get_local_text("settings-developer.developer-mode-description"),
                Switch {
                    active: config.developer.developer_mode,
                    onflipped: move |value| {
                        config.set_developer_mode(value);
                        // cause a re-render
                        state.write();
                    },
                }
            },
            SettingSection {
                section_label: get_local_text("settings-developer.open-codebase"),
                section_description: get_local_text("settings-developer.open-codebase-description"),
                Button {
                    text: get_local_text("settings-developer.open-codebase"),
                    aria_label: "open-codebase-button".into(),
                    appearance: Appearance::Secondary,
                    icon: Icon::CodeBracketSquare,
                    onpress: |_| {
                        let _ = open::that("https://github.com/Satellite-im/Uplink");
                    }
                }
            },
            SettingSection {
                section_label: get_local_text("settings-developer.open-cache"),
                section_description: get_local_text("settings-developer.open-cache-description"),
                Button {
                    text: get_local_text("settings-developer.open-cache-folder"),
                    aria_label: "open-cache-folder-button".into(),
                    appearance: Appearance::Secondary,
                    icon: Icon::FolderOpen,
                    onpress: |_| {
                        let cache_path = dirs::home_dir()
                            .unwrap_or_default()
                            .join(".uplink/")
                            .into_os_string()
                            .into_string()
                            .unwrap_or_default();
                        let _ = opener::open(cache_path);
                    }
                }
            },
            SettingSection {
                section_label: get_local_text("settings-developer.compress-download-cache"),
                section_description: get_local_text("settings-developer.compress-download-cache-description"),
                Button {
                    text: get_local_text("settings-developer.compress"),
                    aria_label: "compress-button".into(),
                    appearance: Appearance::Secondary,
                    icon: Icon::ArchiveBoxArrowDown,
                    onpress: |_| {
                    }
                }
            },
            SettingSection {
                section_label: get_local_text("settings-developer.clear-cache"),
                section_description: get_local_text("settings-developer.clear-cache-description"),
                Button {
                    text: get_local_text("settings-developer.clear"),
                    aria_label: "clear-button".into(),
                    appearance: Appearance::Danger,
                    icon: Icon::Trash,
                    onpress: move |_| {
                        state.write().clear();
                    }
                }
            }
            SettingSection {
                section_label: get_local_text("settings-developer.debug-logger"),
                section_description: get_local_text("settings-developer.debug-logger-description"),
                Button {
                    text: get_local_text("settings-developer.open-debug-logger"),
                    aria_label: "debug-logger-button".into(),
                    appearance: Appearance::Secondary,
                    icon: Icon::CodeBracketSquare,
                    onpress: move |_| {
                        if state.read().ui.current_debug_logger.is_some() {
                            state.write().mutate(Action::ClearDebugLogger(window.clone()));
                            return;
                        }
                        let drop_handler = WindowDropHandler::new(WINDOW_CMD_CH.tx.clone());
                        let logger_debug = VirtualDom::new_with_props(DebugLogger, DebugLoggerProps{
                            _drop_handler: drop_handler,
                        });
                        let window = window.new_window(logger_debug, Default::default());
                        if let Some(wv) = Weak::upgrade(&window) {
                            let id = wv.window().id();
                            state.write().mutate(Action::SetDebugLogger(id));
                        }
                    }
                }
            }
            SettingSection {
                section_label: get_local_text("settings-developer.save-logs-to-file"),
                section_description: get_local_text("settings-developer.save-logs-to-file-description"),
                Switch {
                    active: logger::get_save_to_file(),
                    onflipped: move |value| {
                        logger::set_save_to_file(value);
                    },
                }
            }
        }
    ))
}

pub struct WindowDropHandler {
    cmd_tx: WindowManagerCmdTx,
}

impl PartialEq for WindowDropHandler {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl WindowDropHandler {
    pub fn new(cmd_tx: WindowManagerCmdTx) -> Self {
        Self { cmd_tx }
    }
}

impl Drop for WindowDropHandler {
    fn drop(&mut self) {
        if let Err(e) = self.cmd_tx.send(WindowManagerCmd::CloseDebugLogger) {
            log::warn!("WindowDropHandler failed to send msg: {}", e);
        }
    }
}
