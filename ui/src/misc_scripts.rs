use super::*;

const PRISM_SCRIPT: &str = include_str!("../extra/assets/scripts/prism.js");
pub const PRISM_STYLE: &str = include_str!("../extra/assets/styles/prism.css");
pub const PRISM_THEME: &str = include_str!("../extra/assets/styles/prism-one-dark.css");

pub fn PrismScripts(cx: Scope) -> Element {
    let prism_path = use_prism_path(cx);

    render! {
        script { "{PRISM_SCRIPT}" },
        script { "{prism_path}" },
    }
}

fn use_prism_path(cx: &ScopeState) -> &str {
    cx.use_hook(|| {
        format!(
            r"Prism.plugins.autoloader.languages_path = '{}';",
            get_prism_path().to_string_lossy()
        )
    })
}

fn get_prism_path() -> PathBuf {
    if STATIC_ARGS.production_mode {
        if cfg!(target_os = "windows") {
            STATIC_ARGS.dot_uplink.join("prism_langs")
        } else {
            get_extras_dir().unwrap_or_default().join("prism_langs")
        }
    } else {
        PathBuf::from("ui").join("extra").join("prism_langs")
    }
}
