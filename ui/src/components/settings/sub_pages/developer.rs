use dioxus::prelude::*;
use kit::{elements::{switch::Switch, Appearance, button::Button}, icons::Icon};

use crate::{components::settings::SettingSection, state::State, utils::language::get_local_text};


#[allow(non_snake_case)]
pub fn DeveloperSettings(cx: Scope) -> Element {
    let state = use_context::<State>(&cx).unwrap();

    cx.render(rsx!(
        div {
            id: "settings-developer",
            SettingSection {
                section_label: get_local_text("settings-developer.developer-mode"),
                section_description: get_local_text("settings-developer.developer-mode-description"),
                Switch {
                    
                }
            },
            SettingSection {
                section_label: get_local_text("settings-developer.open-codebase"),
                section_description: get_local_text("settings-developer.open-codebase-description"),
                Button {
                    text: get_local_text("settings-developer.open-codebase"),
                    appearance: Appearance::Secondary,
                    icon: Icon::CodeBracketSquare,
                    onpress: |_| {
                        let _ = open::that("https://github.com/Satellite-im/Uplink-UI_Kit/tree/main/uplink_skeleton");
                    }
                }
            },
            SettingSection {
                section_label: get_local_text("settings-developer.open-cache"),
                section_description: get_local_text("settings-developer.open-cache-description"),
                Button {
                    text: get_local_text("settings-developer.open-cache-folder"),
                    appearance: Appearance::Secondary,
                    icon: Icon::FolderOpen,
                    onpress: |_| {
                        let cache_path = dirs::home_dir()
                            .unwrap_or_default()
                            .join(".uplink/")
                            .into_os_string()
                            .into_string()
                            .unwrap_or_default();
                        let _ = opener::open(&cache_path);
                    }
                }
            },
            SettingSection {
                section_label: get_local_text("settings-developer.compress-download-cache"),
                section_description: get_local_text("settings-developer.compress-download-cache-description"),
                Button {
                    text: get_local_text("settings-developer.compress"),
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
                    appearance: Appearance::Danger,
                    icon: Icon::Trash,
                    onpress: move |_| {
                        state.write().clear();
                    }
                }
            }
        }
    ))
}
