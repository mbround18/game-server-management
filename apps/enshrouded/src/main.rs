mod environment;
mod game_settings;
mod utils;

use crate::environment::name;
use clap::{Parser, Subcommand};
use gsm_cron::{begin_cron_loop, register_job};
use gsm_instance::{Instance, InstanceConfig};
use gsm_monitor::LogRules;
use gsm_notifications::notifications::{StandardServerEvents, send_notifications};
use gsm_shared::{fetch_var, is_env_var_truthy};
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

#[derive(Parser)]
#[command(
    name = "enshrouded",
    version = "1.1",
    about = "Manage Enshrouded Server"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install {
        #[arg(long, default_value = "/home/steam/enshrouded")]
        path: PathBuf,
    },
    /// Start the server only (without monitoring jobs)
    Start,
    /// Monitor the server: start the server and then run scheduled jobs and watch logs.
    Monitor {
        #[arg(long)]
        update_job: bool,
        #[arg(long)]
        restart_job: bool,
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

    fn setup_configuration(game_root: &Path) {
        let config_path = game_root.join("enshrouded_server.json");
        debug!("Loading or creating config at: {:?}", config_path);
        game_settings::load_or_create_config(&config_path);
        debug!("Config load or creation completed.");
    }

    // Set the TZ environment variable to your desired timezone.
    #[cfg(unix)]
    unsafe {
        env::set_var("TZ", fetch_var("TZ", "America/Los_Angeles"));
    }

    let cli = Cli::parse();
    let instance_config = InstanceConfig {
        app_id: 2278520, // Enshrouded Steam App ID
        name: name(),
        command: "enshrouded_server.exe".to_string(),
        install_args: vec![],
        launch_args: vec![],
        force_windows: true,
        working_dir: PathBuf::from("/home/steam/enshrouded"),
    };
    debug!("Instance configuration set: {:?}", instance_config);

    // Use tokio::sync::Mutex for async locking.
    let instance = Arc::new(Mutex::new(Instance::new(instance_config)));
    debug!("Instance created and wrapped in Arc<Mutex<>>");

    match cli.command {
        Commands::Install { path } => {
            info!("Installing Enshrouded server to: {:?}", path);
            debug!("Acquiring lock for installation...");
            let inst = instance.lock().await;
            if let Err(e) = inst.install() {
                error!("Installation failed: {}", e);
            } else {
                debug!("Installation successful.");
                setup_configuration(&path);
                info!("Enshrouded server installed successfully at: {:?}", path);
            }
        }
        Commands::Start => {
            info!("Starting server...");
            let inst = instance.lock().await;
            setup_configuration(&inst.config.working_dir);
            if let Err(e) = inst.start() {
                error!("Failed to start server: {}", e);
            } else {
                debug!("Server started successfully.");
            }
        }
        Commands::Monitor {
            update_job,
            restart_job,
        } => {
            // Start your server and schedule jobs as needed...
            // Then, to watch the logs:
            let working_dir = {
                let inst = instance.lock().await;
                inst.config.working_dir.clone()
            };

            let rules = LogRules::default();

            if env::var("WEBHOOK_URL").is_ok() {
                rules.add_rule(
                    |line| line.contains("[Session] 'HostOnline' (up)!"),
                    |_| {
                        send_notifications(StandardServerEvents::Started)
                            .expect("Failed to send webhook event! Invalid url?")
                    },
                    false,
                    None,
                );

                rules.add_rule(
                    |line| line.contains("logged in with Permissions:"),
                    |line| match utils::extract_player_joined_name(line) {
                        Some(name) => send_notifications(StandardServerEvents::PlayerJoined(name))
                            .expect("Failed to send webhook event! Invalid url?"),
                        None => error!("Failed to extract player name from:\n{line}"),
                    },
                    false,
                    None,
                );

                rules.add_rule(
                    |line| line.contains("[server] Remove Entity for Player"),
                    |line| match utils::extract_player_left_name(line) {
                        Some(name) => send_notifications(StandardServerEvents::PlayerLeft(name))
                            .expect("Failed to send webhook event! Invalid url?"),
                        None => error!("Failed to extract player name from:\n{line}"),
                    },
                    false,
                    None,
                );
            }

            // Start monitoring the instance log files.
            gsm_monitor::start_instance_log_monitor(working_dir, rules);

            if update_job || is_env_var_truthy("AUTO_UPDATE") {
                debug!("Auto-update job condition met.");
                let update_schedule = fetch_var("AUTO_UPDATE_SCHEDULE", "0 3 * * *");
                debug!("Auto-update schedule: {}", update_schedule);
                let instance_clone = Arc::clone(&instance);
                register_job("auto-update", &update_schedule, move || {
                    debug!("Auto-update job triggered.");
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
                        } else {
                            debug!("No updates available during auto-update check.");
                        }
                    });
                });
            } else {
                debug!("Auto-update job not enabled.");
            }

            if restart_job || is_env_var_truthy("SCHEDULED_RESTART") {
                debug!("Scheduled restart job condition met.");
                let restart_schedule = fetch_var("SCHEDULED_RESTART_SCHEDULE", "0 4 * * *");
                debug!("Scheduled restart schedule: {}", restart_schedule);
                let instance_clone = Arc::clone(&instance);
                register_job("scheduled-restart", &restart_schedule, move || {
                    debug!("Scheduled restart job triggered.");
                    let instance_clone_inner = Arc::clone(&instance_clone);
                    tokio::spawn(async move {
                        let inst = instance_clone_inner.lock().await;
                        warn!("Restarting server...");
                        if let Err(e) = inst.restart() {
                            error!("Failed to restart server: {}", e);
                        }
                    });
                });
            } else {
                debug!("Scheduled restart job not enabled.");
            }

            debug!("Entering cron loop (monitoring logs and scheduled tasks)...");
            begin_cron_loop().await;
            debug!("Cron loop ended.");
        }
        Commands::Stop => {
            let webhook_enabled = env::var("WEBHOOK_URL").is_ok();
            if webhook_enabled && let Ok(delay_str) = env::var("STOP_DELAY") {
                match delay_str.parse::<u64>() {
                    Ok(delay_sec) => {
                        send_notifications(StandardServerEvents::Stopping)
                            .expect("Failed to send webhook event! Invalid url?");
                        tokio::time::sleep(Duration::from_secs(delay_sec)).await;
                    }
                    Err(_) => {
                        error!("Invalid STOP_DELAY value: {}", delay_str);
                    }
                }
            }

            warn!("Stopping Enshrouded server...");
            debug!("Acquiring lock to stop the server...");

            let inst = instance.lock().await;
            match inst.stop() {
                Err(e) => {
                    error!("Failed to stop: {}", e);
                }
                Ok(_) => {
                    if webhook_enabled {
                        send_notifications(StandardServerEvents::Stopped)
                            .expect("Failed to send webhook event! Invalid url?");
                    }
                    debug!("Server stopped successfully.");
                }
            }
        }
        Commands::Restart => {
            warn!("Restarting Enshrouded server...");
            debug!("Acquiring lock to restart the server...");
            let inst = instance.lock().await;
            if let Err(e) = inst.restart() {
                error!("Failed to restart server: {}", e);
            } else {
                debug!("Server restarted successfully.");
            }
        }
        Commands::Update { check } => {
            debug!("Update command initiated with check = {}", check);
            let inst = instance.lock().await;
            if check {
                debug!("Performing update check...");
                if inst.update_available() {
                    info!("Update available!");
                    exit(1);
                } else {
                    info!("Server is up to date.");
                    exit(0);
                }
            } else {
                info!("Checking for updates without enforcing check flag...");
                if inst.update_available() {
                    warn!("Update available! Updating...");
                    if let Err(e) = inst.update() {
                        error!("Update failed: {}", e);
                    } else {
                        debug!("Update applied successfully.");
                    }
                } else {
                    debug!("Server is up to date; no update needed.");
                }
            }
        }
    }
}
