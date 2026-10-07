use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Context;
use clap::{Parser, Subcommand};
use gtk::glib;
use gtk::prelude::*;
use gtk::Application;

use omakeys::board::client::{BoardClient, HidBoardClient, Keymap};
use omakeys::board::loremipsum36;
use omakeys::board::HostEvent;
use omakeys::core::{load_config, load_overrides, save_config, AppConfig, OverlayCore};
use omakeys::ipc::{self, IpcCommand};
use omakeys::ui::{apply_theme_icons, load_omarchy_theme, OverlayWindow};

#[derive(Parser)]
#[command(name = "omakeys", about = "Omakeys live layer map")]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run the overlay daemon (default)
    Run,
    /// Toggle overlay visibility via IPC
    Toggle,
    /// Open Omarchy bar settings panel
    Settings,
    /// Print config.toml as JSON
    ConfigPrint,
    /// Write config from JSON, then reload daemon if running
    ConfigApply { json: String },
    /// Check daemon socket (expects ok)
    Ping,
    /// Quit the running daemon
    Quit,
}

enum AppMsg {
    Events(Vec<HostEvent>),
    Keymap(Keymap),
    Ipc(IpcCommand),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.cmd.unwrap_or(Command::Run) {
        Command::Run => run_daemon(),
        Command::Toggle => ipc::send_command(IpcCommand::Toggle),
        Command::Settings => {
            let status = std::process::Command::new("omarchy-shell")
                .args(["-q", "omakeys", "toggle"])
                .status()
                .context("omarchy-shell omakeys toggle")?;
            if !status.success() {
                anyhow::bail!("failed to toggle omakeys panel (is Omarchy shell running?)");
            }
            Ok(())
        }
        Command::ConfigPrint => {
            let cfg = load_config();
            println!("{}", serde_json::to_string_pretty(&cfg)?);
            Ok(())
        }
        Command::ConfigApply { json } => {
            let cfg: AppConfig = serde_json::from_str::<AppConfig>(&json)?.normalized();
            save_config(&cfg)?;
            let _ = ipc::send_command(IpcCommand::Reload);
            Ok(())
        }
        Command::Ping => ipc::send_command(IpcCommand::Ping),
        Command::Quit => ipc::send_command(IpcCommand::Quit),
    }
}

fn run_daemon() -> anyhow::Result<()> {
    let app = Application::builder()
        .application_id("org.omakeys.app")
        .build();

    app.connect_activate(|app| {
        let profile = loremipsum36();
        let config = load_config();
        let overrides = load_overrides();
        let core = OverlayCore::new(profile.clone(), config.clone(), overrides);
        let force_hidden = core.force_hidden_handle();
        let overlay = OverlayWindow::new(app);
        let theme = load_omarchy_theme();
        if let Err(e) = apply_theme_icons(&theme) {
            eprintln!("omakeys: theme icons: {e}");
        }
        overlay.set_theme(theme);

        let (tx, rx) = mpsc::channel::<AppMsg>();
        let profile_bg = profile;
        let tx_board = tx.clone();
        thread::spawn(move || board_loop(profile_bg, tx_board));

        let (ipc_tx, ipc_rx) = mpsc::channel::<IpcCommand>();
        if let Err(e) = ipc::spawn_server(ipc_tx, force_hidden) {
            eprintln!("omakeys: IPC server: {e}");
        }

        let tx_ipc = tx.clone();
        thread::spawn(move || {
            while let Ok(cmd) = ipc_rx.recv() {
                let _ = tx_ipc.send(AppMsg::Ipc(cmd));
            }
        });

        let overlay = Rc::new(overlay);
        let core = Rc::new(RefCell::new(core));
        let rx = Rc::new(RefCell::new(rx));
        let app_rc = app.clone();
        let config_cell = Rc::new(RefCell::new(config));

        glib::timeout_add_local(Duration::from_millis(16), move || {
            let mut should_quit = false;
            {
                let rx = rx.borrow();
                loop {
                    match rx.try_recv() {
                        Ok(AppMsg::Events(events)) => {
                            let now = Instant::now();
                            let mut c = core.borrow_mut();
                            for ev in events {
                                c.handle(ev, now);
                            }
                        }
                        Ok(AppMsg::Keymap(map)) => core.borrow_mut().set_keymap(map),
                        Ok(AppMsg::Ipc(IpcCommand::Toggle)) => {
                            // `force_hidden` already flipped in the IPC thread.
                        }
                        Ok(AppMsg::Ipc(IpcCommand::Settings)) => {
                            // GTK settings removed; panel is the Omarchy bar widget.
                        }
                        Ok(AppMsg::Ipc(IpcCommand::Reload)) => {
                            let cfg = load_config();
                            let ovr = load_overrides();
                            let mut c = core.borrow_mut();
                            c.set_config(cfg.clone());
                            c.set_overrides(ovr);
                            *config_cell.borrow_mut() = cfg;
                            let theme = load_omarchy_theme();
                            if let Err(e) = apply_theme_icons(&theme) {
                                eprintln!("omakeys: theme icons: {e}");
                            }
                            overlay.set_theme(theme);
                        }
                        Ok(AppMsg::Ipc(IpcCommand::Quit)) => {
                            should_quit = true;
                        }
                        Ok(AppMsg::Ipc(IpcCommand::Ping)) => {
                            // Handled in the IPC server thread (reply ok).
                        }
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => break,
                    }
                }
            }

            if should_quit {
                app_rc.quit();
                return glib::ControlFlow::Break;
            }

            core.borrow_mut().tick(Instant::now());
            let frame = core.borrow().frame();
            overlay.apply_frame(&frame);
            glib::ControlFlow::Continue
        });
    });

    // Clap already consumed argv ("run", etc.). Passing process args into GTK
    // makes GIO try to OPEN them as files → CRITICAL "can not open files".
    app.run_with_args(&[] as &[&str]);
    Ok(())
}

fn board_loop(profile: omakeys::board::BoardProfile, tx: mpsc::Sender<AppMsg>) {
    let Ok(mut client) = HidBoardClient::open(&profile) else {
        eprintln!("omakeys: failed to init HID");
        return;
    };
    let mut was_ok = false;
    loop {
        let events = client.poll_events();
        if !events.is_empty() {
            let _ = tx.send(AppMsg::Events(events));
        }
        let ok = client.ok();
        if ok && !was_ok {
            let _ = tx.send(AppMsg::Keymap(client.keymap().clone()));
        }
        was_ok = ok;
        thread::sleep(Duration::from_millis(5));
    }
}
