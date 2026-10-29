const CONFIG_NAME: &str = "config.toml";
const CONFIG_DIR: &str = "gits";

// Omitted actions deserialize to an empty list, which falls back to the default keys.
#[derive(Debug, Clone, Default, ::serde::Deserialize)]
#[serde(default)]
pub struct Keymap {
    pub up: Vec<String>,
    pub down: Vec<String>,
    pub confirm: Vec<String>,
    pub cancel: Vec<String>,
}

#[derive(Debug, ::serde::Deserialize)]
pub struct Config {
    pub keymap: Option<Keymap>,
}

impl Config {
    pub fn load() -> ::anyhow::Result<Self> {
        let path = config_path()?;
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}

/// Parse the config file and render it as TOML, annotated with how each
/// entry is applied (defaults, ignored keys, unknown fields).
pub fn describe() -> ::anyhow::Result<String> {
    let path = config_path()?;
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => Some(content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => anyhow::bail!("{}: {e}", path.display()),
    };
    render(&path, content.as_deref())
}

/// `content` is `None` when the config file does not exist.
fn render(path: &std::path::Path, content: Option<&str>) -> ::anyhow::Result<String> {
    let Some(content) = content else {
        let mut out = format!("# {} (not found; default keys are used)\n", path.display());
        render_keymap(&mut out, &Keymap::default());
        return Ok(out);
    };

    let config: Config = toml::from_str(content)
        .map_err(|e| anyhow::anyhow!("{} (default keys are used)\n{e}", path.display()))?;
    let table: toml::Table = toml::from_str(content)?;

    let mut out = format!("# {}\n", path.display());
    for key in table.keys().filter(|&k| k != "keymap") {
        out.push_str(&format!("# unknown field: {key} (ignored)\n"));
    }
    if config.keymap.is_none() {
        out.push_str("# no [keymap] section; default keys are used\n");
    }

    render_keymap(&mut out, &config.keymap.unwrap_or_default());

    let known = ["up", "down", "confirm", "cancel"];
    if let Some(toml::Value::Table(km)) = table.get("keymap") {
        for key in km.keys().filter(|k| !known.contains(&k.as_str())) {
            out.push_str(&format!("# unknown field: keymap.{key} (ignored)\n"));
        }
    }

    Ok(out)
}

/// Render `[keymap]`, showing the default keys for actions that fall back to them.
fn render_keymap(out: &mut String, keymap: &Keymap) {
    use crate::selector::DEFAULT_KEYS;

    out.push_str("\n[keymap]\n");
    let actions = [
        ("up", &keymap.up, DEFAULT_KEYS.up),
        ("down", &keymap.down, DEFAULT_KEYS.down),
        ("confirm", &keymap.confirm, DEFAULT_KEYS.confirm),
        ("cancel", &keymap.cancel, DEFAULT_KEYS.cancel),
    ];
    for (name, keys, defaults) in actions {
        let (valid, invalid): (Vec<&str>, Vec<&str>) = keys
            .iter()
            .map(String::as_str)
            .partition(|k| crate::selector::parse_binding(k).is_some());

        let mut notes = Vec::new();
        let shown = if valid.is_empty() {
            notes.push("default".to_owned());
            defaults.to_vec()
        } else {
            valid
        };
        if !invalid.is_empty() {
            notes.push(format!("ignored: {}", quote_list(&invalid)));
        }

        out.push_str(&format!("{name} = [{}]", quote_list(&shown)));
        if !notes.is_empty() {
            out.push_str(&format!(" # {}", notes.join(", ")));
        }
        out.push('\n');
    }
}

fn quote_list(keys: &[&str]) -> String {
    keys.iter()
        .map(|k| format!("{k:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::render;

    fn run(content: &str) -> String {
        render(std::path::Path::new("config.toml"), Some(content)).unwrap()
    }

    #[test]
    fn marks_defaults_and_ignored_keys() {
        let out = run("[keymap]\nup = [\"k\", \"ctrl-p\", \"bogus\"]\ncancel = []\n");
        assert_eq!(
            out,
            "# config.toml\n\
             \n\
             [keymap]\n\
             up = [\"k\", \"ctrl-p\"] # ignored: \"bogus\"\n\
             down = [\"j\", \"down\"] # default\n\
             confirm = [\"enter\", \"l\"] # default\n\
             cancel = [\"esc\", \"h\", \"q\", \"ctrl-c\"] # default\n"
        );
    }

    #[test]
    fn shows_defaults_when_file_is_missing() {
        let out = render(std::path::Path::new("config.toml"), None).unwrap();
        assert_eq!(
            out,
            "# config.toml (not found; default keys are used)\n\
             \n\
             [keymap]\n\
             up = [\"k\", \"up\"] # default\n\
             down = [\"j\", \"down\"] # default\n\
             confirm = [\"enter\", \"l\"] # default\n\
             cancel = [\"esc\", \"h\", \"q\", \"ctrl-c\"] # default\n"
        );
    }

    #[test]
    fn default_keys_are_valid_bindings() {
        use crate::selector::{DEFAULT_KEYS, parse_binding};
        let all = [
            DEFAULT_KEYS.up,
            DEFAULT_KEYS.down,
            DEFAULT_KEYS.confirm,
            DEFAULT_KEYS.cancel,
        ];
        for key in all.concat() {
            assert!(parse_binding(key).is_some(), "{key}");
        }
    }

    #[test]
    fn reports_unknown_fields() {
        let out = run("theme = 1\n[keymap]\nconfirmm = [\"enter\"]\n");
        assert!(out.contains("# unknown field: theme (ignored)\n"));
        assert!(out.contains("# unknown field: keymap.confirmm (ignored)\n"));
    }

    #[test]
    fn reports_missing_keymap_section() {
        assert!(run("").contains("# no [keymap] section; default keys are used\n"));
    }

    #[test]
    fn reports_parse_errors() {
        let err = render(
            std::path::Path::new("config.toml"),
            Some("[keymap]\nup = \"k\"\n"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("default keys are used"));
    }
}

pub fn config_path() -> ::anyhow::Result<std::path::PathBuf> {
    let mut path =
        dirs::config_dir().ok_or_else(|| anyhow::anyhow!("could not locate config directory"))?;
    path.push(CONFIG_DIR);
    path.push(CONFIG_NAME);
    Ok(path)
}
