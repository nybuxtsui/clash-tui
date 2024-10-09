use std::{env, path::PathBuf, sync::{LazyLock, RwLock}};

#[derive(Default, Clone)]
pub struct Config {
    pub host: String,
    pub key: String,
}

pub fn get_config() -> Config {
    CONFIG.read().unwrap().clone()
}

pub static CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| {
    RwLock::new(Config {
        host: "127.0.0.1:9090".to_string(),
        key: "".to_string(),
    })
});

pub fn load_config() -> anyhow::Result<Config> {
    let ini = "clash-tui.ini";
    let default_host = "127.0.0.1".to_string();
    let default_key = String::new();

    let mut path = env::current_exe()?.parent().unwrap_or(PathBuf::from("").as_path()).join(ini);
    if !path.exists() {
        path = env::current_dir()?.join(ini);
    }

    if !path.exists() {
        return Ok(Config { host: default_host, key: default_key })
    }
    let settings = config::Config::builder()
        .add_source(config::File::with_name(path.to_str().unwrap_or(ini)))
        .build()?;

    let args: Vec<String> = env::args().collect();
    let key = if args.len() > 1 {
        (format!("{}.host", &args[1]), format!("{}.key", &args[1]))
    } else {
        ("host".into(), "key".into())
    };

    let host: String = settings.get(&key.0).unwrap_or(default_host);
    let key: String = settings.get(&key.1).unwrap_or(default_key);
    Ok(Config { host, key })
}
