#![allow(clippy::all, dead_code, unused_imports)]

#[path = "../app.rs"]
mod app;
#[cfg(windows)]
#[path = "../boot.rs"]
mod boot;
#[cfg(not(windows))]
#[path = "../boot_stub.rs"]
mod boot;
#[path = "../categories.rs"]
mod categories;
#[path = "../cli.rs"]
mod cli;
#[path = "../config.rs"]
mod config;
#[cfg(windows)]
#[path = "../drains.rs"]
mod drains;
#[cfg(not(windows))]
#[path = "../drains_stub.rs"]
mod drains;
#[path = "../hwtemp.rs"]
mod hwtemp;
#[path = "../icons.rs"]
mod icons;
#[path = "../knowledge.rs"]
mod knowledge;
#[path = "../metrics.rs"]
mod metrics;
#[path = "../procs.rs"]
mod procs;
#[path = "../sampler.rs"]
mod sampler;
#[path = "../signature.rs"]
mod signature;
#[cfg(windows)]
#[path = "../sys.rs"]
mod sys;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        if launch_gui() {
            return;
        }
        std::process::exit(cli::run(vec!["help".into()]));
    }
    std::process::exit(cli::run(args));
}

fn launch_gui() -> bool {
    let Ok(current) = std::env::current_exe() else {
        return false;
    };
    let Some(dir) = current.parent() else {
        return false;
    };
    let names: &[&str] = if cfg!(windows) {
        &["ramdog-gui.exe", "ramdog.exe"]
    } else {
        &["ramdog-gui", "ramdog"]
    };
    names
        .iter()
        .map(|name| dir.join(name))
        .find(|path| path.is_file())
        .map(|path| std::process::Command::new(path).spawn().is_ok())
        .unwrap_or(false)
}
