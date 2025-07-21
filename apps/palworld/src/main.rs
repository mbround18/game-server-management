mod environment;
mod game_settings;
mod utils;

use crate::environment::name;
use clap::{Parser, Subcommand, arg};
use gsm_cron::{begin_cron_loop, register_job};
use gsm_instance::{Instance, InstanceConfig};
use gsm_monitor::LogRules;
use gsm_notifications::notifications::{StandardServerEvents, send_notifications};
use gsm_shared::{fetch_var, is_env_var_truthy};
use std::env;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

#[derive(Parser)]
#[command(name = "palworld", version = "1.0", about = "Manage Palworld Server")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install {
        #[arg(long, default_value = "/home/steam/palworld")]
        path: PathBuf,
    },
    Start,
    Monitor {
        #[arg(long)]
        update_job: bool,
    },
    Stop,
    Restart,
    Update {
        #[arg(long)]
        check: bool,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    debug!("Tracing subscriber initialized.");

    let cli = Cli::parse();
    let instance_config = InstanceConfig {
        app_id: 2394010, // Palworld Steam App ID
        name: name(),
        command: "/bin/bash".to_string(),
        install_args: vec![],
        launch_args: {
            let mut args = vec!["./PalServer.sh".to_string()];

            if let Ok(public_ip) = env::var("PUBLIC_IP") {
                args.push(format!("-publicip={public_ip}"));
            }

            if let Some(public_port) = env::var("PORT").ok().or(Some("8211".to_string())) {
                args.push(format!("-port={public_port}"));
            }

            if let Some(public_port) = env::var("PUBLIC_PORT").ok().or(Some("8211".to_string())) {
                args.push(format!("-publicport={public_port}"));
            }

            if is_env_var_truthy("PUBLIC_LOBBY") {
                args.push("-publiclobby".to_string());
            }

            if is_env_var_truthy("MULTITHREADING") {
                args.push("-useperfthreads".to_string());
                args.push("-NoAsyncLoadingThread".to_string());
                args.push("-UseMultithreadForDS".to_string());
            }

            args
        },
        force_windows: false,
        working_dir: PathBuf::from("/home/steam/palworld"),
    };
    debug!("Instance configuration set: {:?}", instance_config);

    let instance = Arc::new(Mutex::new(Instance::new(instance_config)));
    debug!("Instance created and wrapped in Arc<Mutex<>>");

    match cli.command {
        Commands::Install { path } => {
            info!("Installing Palworld server to: {:?}", path);
            let inst = instance.lock().await;
            if let Err(e) = inst.install() {
                error!("Installation failed: {}", e);
            } else {
                debug!("Installation successful.");
                let config_path = path.join("Pal/Saved/Config/LinuxServer/PalWorldSettings.ini");
                game_settings::load_or_create_config(&config_path);
            }
        }
        Commands::Start => {
            info!("Starting server...");
            let inst = instance.lock().await;
            if let Err(e) = inst.start() {
                error!("Failed to start server: {}", e);
            }
        }
        Commands::Monitor { update_job } => {
            let working_dir = {
                let inst = instance.lock().await;
                inst.config.working_dir.clone()
            };

            let rules = LogRules::default();

            if env::var("WEBHOOK_URL").is_ok() {
                rules.add_rule(
                    |line| line.contains("Running Palworld dedicated server on"),
                    |_| {
                        send_notifications(StandardServerEvents::Started)
                            .expect("Failed to send webhook event! Invalid url?")
                    },
                    false,
                    None,
                );

                rules.add_rule(
                    |line| line.contains("joined the server."),
                    |line| match utils::extract_player_joined_name(line) {
                        Some(name) => send_notifications(StandardServerEvents::PlayerJoined(name))
                            .expect("Failed to send webhook event! Invalid url?"),
                        None => error!("Failed to extract player name from:\n{line}"),
                    },
                    false,
                    None,
                );

                rules.add_rule(
                    |line| line.contains("left the server."),
                    |line| match utils::extract_player_left_name(line) {
                        Some(name) => send_notifications(StandardServerEvents::PlayerLeft(name))
                            .expect("Failed to send webhook event! Invalid url?"),
                        None => error!("Failed to extract player name from:\n{line}"),
                    },
                    false,
                    None,
                );
            }

            gsm_monitor::start_instance_log_monitor(working_dir, rules);

            if update_job || is_env_var_truthy("AUTO_UPDATE") {
                let update_schedule = fetch_var("AUTO_UPDATE_SCHEDULE", "0 3 * * *");
                let instance_clone = Arc::clone(&instance);
                register_job("auto-update", &update_schedule, move || {
                    let instance_clone_inner = Arc::clone(&instance_clone);
                    tokio::spawn(async move {
                        let inst = instance_clone_inner.lock().await;
                        if inst.update_available() {
                            warn!("Update available! Stopping server...");
                            if let Err(e) = inst.stop() {
                                error!("Failed to stop server: {}", e);
                                return;
                            }
                            info!("Updating server...");
                            if let Err(e) = inst.update() {
                                error!("Update failed: {}", e);
                                return;
                            }
                            info!("Restarting server...");
                            if let Err(e) = inst.start() {
                                error!("Failed to start server: {}", e);
                            }
                        }
                    });
                });
            }

            debug!("Entering cron loop (monitoring logs and scheduled tasks)...");
            begin_cron_loop().await;
        }
        Commands::Stop => {
            warn!("Stopping Palworld server...");
            let inst = instance.lock().await;
            match inst.stop() {
                Err(e) => {
                    error!("Failed to stop: {}", e);
                }
                Ok(_) => {
                    if env::var("WEBHOOK_URL").is_ok() {
                        send_notifications(StandardServerEvents::Stopped)
                            .expect("Failed to send webhook event! Invalid url?");
                    }
                    debug!("Server stopped successfully.");
                }
            }
        }
        Commands::Restart => {
            warn!("Restarting Palworld server...");
            let inst = instance.lock().await;
            if let Err(e) = inst.restart() {
                error!("Failed to restart server: {}", e);
            }
        }
        Commands::Update { check } => {
            let inst = instance.lock().await;
            if check {
                if inst.update_available() {
                    info!("Update available!");
                    exit(1);
                } else {
                    info!("Server is up to date.");
                    exit(0);
                }
            } else if inst.update_available() {
                warn!("Update available! Updating...");
                if let Err(e) = inst.update() {
                    error!("Update failed: {}", e);
                }
            }
        }
    }
}
