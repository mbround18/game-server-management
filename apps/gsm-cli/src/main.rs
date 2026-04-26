mod environment;

use clap::{Args, CommandFactory, Parser, Subcommand, error::ErrorKind};
use environment::{
    app_id as env_app_id, executable as env_executable, force_windows as env_force_windows,
    install_args as env_install_args, install_path as env_install_path,
    launch_args as env_launch_args, launch_mode as env_launch_mode, name,
};
use gsm_cron::{begin_cron_loop, register_job};
use gsm_instance::{Instance, InstanceConfig, config::LaunchMode};
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(
    name = "gsm-cli",
    version,
    about = "Generic Steam dedicated server manager"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Install(InstallCommand),
    Start(RuntimeCommand),
    Stop(RuntimeCommand),
    Restart(RuntimeCommand),
    Update(UpdateCommand),
    Monitor(MonitorCommand),
}

#[derive(Args, Debug, Clone)]
struct SharedOptions {
    #[arg(long)]
    app_id: Option<u32>,
    #[arg(long)]
    install_path: Option<PathBuf>,
    #[arg(long)]
    force_windows: bool,
    #[arg(long, value_name = "native|wine|proton")]
    launch_mode: Option<String>,
    #[arg(long)]
    executable: Option<String>,
    #[arg(long = "install-arg")]
    install_args: Vec<String>,
    #[arg(long = "launch-arg")]
    launch_args: Vec<String>,
}

#[derive(Args, Debug, Clone)]
struct InstallCommand {
    #[command(flatten)]
    shared: SharedOptions,
}

#[derive(Args, Debug, Clone)]
struct RuntimeCommand {
    #[command(flatten)]
    shared: SharedOptions,
}

#[derive(Args, Debug, Clone)]
struct UpdateCommand {
    #[command(flatten)]
    shared: SharedOptions,
    #[arg(long)]
    check: bool,
}

#[derive(Args, Debug, Clone)]
struct MonitorCommand {
    #[command(flatten)]
    shared: SharedOptions,
    #[arg(long)]
    update_job: bool,
}

#[derive(Debug, Clone)]
struct ResolvedOptions {
    app_id: u32,
    install_path: PathBuf,
    executable: Option<String>,
    force_windows: bool,
    launch_mode: LaunchMode,
    install_args: Vec<String>,
    launch_args: Vec<String>,
}

impl SharedOptions {
    fn resolve(&self, require_executable: bool) -> Result<ResolvedOptions, clap::Error> {
        let app_id = self.app_id.or_else(env_app_id).ok_or_else(|| {
            Cli::command().error(
                ErrorKind::MissingRequiredArgument,
                "missing APP_ID or --app-id",
            )
        })?;

        let install_path = self
            .install_path
            .clone()
            .or_else(env_install_path)
            .ok_or_else(|| {
                Cli::command().error(
                    ErrorKind::MissingRequiredArgument,
                    "missing INSTALL_PATH or --install-path",
                )
            })?;

        let executable = self.executable.clone().or_else(env_executable);
        if require_executable && executable.is_none() {
            return Err(Cli::command().error(
                ErrorKind::MissingRequiredArgument,
                "missing EXECUTABLE/COMMAND or --executable",
            ));
        }

        let launch_mode = self
            .launch_mode
            .as_deref()
            .and_then(environment::parse_launch_mode)
            .or_else(env_launch_mode)
            .unwrap_or_else(|| {
                if self.force_windows || env_force_windows() {
                    LaunchMode::Wine
                } else {
                    LaunchMode::Native
                }
            });

        let install_args = if self.install_args.is_empty() {
            env_install_args()
        } else {
            self.install_args.clone()
        };

        let launch_args = if self.launch_args.is_empty() {
            env_launch_args()
        } else {
            self.launch_args.clone()
        };

        Ok(ResolvedOptions {
            app_id,
            install_path,
            executable,
            force_windows: self.force_windows || env_force_windows(),
            launch_mode,
            install_args,
            launch_args,
        })
    }
}

impl ResolvedOptions {
    fn into_instance_config(self) -> InstanceConfig {
        InstanceConfig {
            app_id: self.app_id,
            name: name(),
            command: self.executable.unwrap_or_default(),
            install_args: self.install_args,
            launch_args: self.launch_args,
            force_windows: self.force_windows,
            working_dir: self.install_path,
            launch_mode: self.launch_mode,
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Install(command) => {
            let resolved = unwrap_or_exit(command.shared.resolve(false));
            let instance = Instance::new(resolved.into_instance_config());

            info!(
                "Installing app {} to {}",
                instance.config.app_id,
                instance.config.working_dir.display()
            );

            if let Err(err) = instance.install() {
                error!("Installation failed: {err}");
                exit(1);
            }
        }
        Commands::Start(command) => {
            let resolved = unwrap_or_exit(command.shared.resolve(true));
            let instance = Instance::new(resolved.into_instance_config());

            if let Err(err) = instance.start() {
                error!("Failed to start server: {err}");
                exit(1);
            }
        }
        Commands::Stop(command) => {
            let resolved = unwrap_or_exit(command.shared.resolve(true));
            let instance = Instance::new(resolved.into_instance_config());

            if let Err(err) = instance.stop() {
                error!("Failed to stop server: {err}");
                exit(1);
            }
        }
        Commands::Restart(command) => {
            let resolved = unwrap_or_exit(command.shared.resolve(true));
            let instance = Instance::new(resolved.into_instance_config());

            if let Err(err) = instance.restart() {
                error!("Failed to restart server: {err}");
                exit(1);
            }
        }
        Commands::Update(command) => {
            let resolved = unwrap_or_exit(command.shared.resolve(false));
            let instance = Instance::new(resolved.into_instance_config());

            if command.check {
                if instance.update_available() {
                    info!("Update available for app {}", instance.config.app_id);
                    exit(1);
                }

                info!("App {} is up to date", instance.config.app_id);
                exit(0);
            }

            if let Err(err) = instance.update() {
                error!("Update failed: {err}");
                exit(1);
            }
        }
        Commands::Monitor(command) => {
            let resolved = unwrap_or_exit(command.shared.resolve(false));
            let instance = Arc::new(Mutex::new(Instance::new(resolved.into_instance_config())));

            let working_dir = {
                let instance = instance.lock().await;
                instance.config.working_dir.clone()
            };

            gsm_monitor::start_instance_log_monitor(working_dir, gsm_monitor::LogRules::default());

            if command.update_job || gsm_shared::is_env_var_truthy("AUTO_UPDATE") {
                let schedule = gsm_shared::fetch_var("AUTO_UPDATE_SCHEDULE", "0 3 * * *");
                let update_instance = Arc::clone(&instance);

                register_job("auto-update", &schedule, move || {
                    let update_instance = Arc::clone(&update_instance);
                    tokio::spawn(async move {
                        let instance = update_instance.lock().await;
                        if instance.update_available() {
                            warn!(
                                "Update available for app {}. Applying update.",
                                instance.config.app_id
                            );

                            if let Err(err) = instance.update() {
                                error!("Auto-update failed: {err}");
                            }
                        }
                    });
                });
            }

            begin_cron_loop().await;
        }
    }
}

fn unwrap_or_exit<T>(result: Result<T, clap::Error>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => error.exit(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[cfg(unix)]
    fn write_executable_script(path: &Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;

        fs::write(path, body).unwrap();
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    #[test]
    fn cli_values_override_environment_values() {
        let _lock = env_lock().lock().unwrap_or_else(|error| error.into_inner());

        unsafe {
            env::set_var("APP_ID", "1234");
            env::set_var("INSTALL_PATH", "/tmp/from-env");
            env::set_var("EXECUTABLE", "env-server");
        }

        let options = SharedOptions {
            app_id: Some(4321),
            install_path: Some(PathBuf::from("/tmp/from-cli")),
            force_windows: false,
            launch_mode: Some(String::from("proton")),
            executable: Some(String::from("cli-server")),
            install_args: vec![String::from("+beta")],
            launch_args: vec![String::from("-log")],
        };

        let resolved = options.resolve(true).unwrap();

        assert_eq!(resolved.app_id, 4321);
        assert_eq!(resolved.install_path, PathBuf::from("/tmp/from-cli"));
        assert_eq!(resolved.executable.as_deref(), Some("cli-server"));
        assert!(matches!(resolved.launch_mode, LaunchMode::Proton));
        assert_eq!(resolved.install_args, vec!["+beta"]);
        assert_eq!(resolved.launch_args, vec!["-log"]);

        unsafe {
            env::remove_var("APP_ID");
            env::remove_var("INSTALL_PATH");
            env::remove_var("EXECUTABLE");
        }
    }

    #[test]
    fn executable_is_required_for_runtime_commands() {
        let _lock = env_lock().lock().unwrap_or_else(|error| error.into_inner());

        unsafe {
            env::set_var("APP_ID", "2278520");
            env::set_var("INSTALL_PATH", "/tmp/server");
            env::remove_var("EXECUTABLE");
            env::remove_var("COMMAND");
        }

        let options = SharedOptions {
            app_id: None,
            install_path: None,
            force_windows: false,
            launch_mode: None,
            executable: None,
            install_args: Vec::new(),
            launch_args: Vec::new(),
        };

        let error = options.resolve(true).unwrap_err();
        let rendered = error.to_string();
        assert!(rendered.contains("missing EXECUTABLE/COMMAND or --executable"));

        unsafe {
            env::remove_var("APP_ID");
            env::remove_var("INSTALL_PATH");
        }
    }

    #[cfg(unix)]
    #[test]
    fn install_uses_env_values_when_cli_values_are_missing() {
        let _lock = env_lock().lock().unwrap_or_else(|error| error.into_inner());
        let temp_dir = tempdir().unwrap();
        let args_path = temp_dir.path().join("args.txt");
        let script_path = temp_dir.path().join("fake-steamcmd.sh");
        let install_dir = temp_dir.path().join("server");
        let script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nexit 0\n",
            args_path.display()
        );
        write_executable_script(&script_path, &script);

        unsafe {
            env::set_var("STEAMCMD_PATH", &script_path);
            env::set_var("APP_ID", "2394010");
            env::set_var("INSTALL_PATH", &install_dir);
            env::set_var("INSTALL_ARGS", "+beta staging");
        }

        let options = SharedOptions {
            app_id: None,
            install_path: None,
            force_windows: false,
            launch_mode: None,
            executable: None,
            install_args: Vec::new(),
            launch_args: Vec::new(),
        };

        let resolved = options.resolve(false).unwrap();
        let instance = Instance::new(resolved.into_instance_config());
        instance.install().unwrap();

        let recorded_args = fs::read_to_string(&args_path).unwrap();
        let lines: Vec<&str> = recorded_args.lines().collect();
        assert_eq!(
            lines[0],
            format!("+force_install_dir {}", install_dir.display())
        );
        assert_eq!(lines[1], "+login anonymous");
        assert_eq!(lines[2], "+app_update 2394010 validate");
        assert_eq!(lines[3], "+beta");
        assert_eq!(lines[4], "staging");
        assert_eq!(lines[5], "+quit");

        unsafe {
            env::remove_var("STEAMCMD_PATH");
            env::remove_var("APP_ID");
            env::remove_var("INSTALL_PATH");
            env::remove_var("INSTALL_ARGS");
        }
    }
}
