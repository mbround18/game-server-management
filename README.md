# Game Server Management

Game Server Management is the culmination of lessons learned from [mbround18/valheim-docker](https://github.com/mbround18/valheim-docker) and its robust Rust code. This repository provides a Docker container enriched with CLI tools for crafting game-specific containers.

## Manifesto

Our mission is clear: **All SteamCMD dedicated servers should be easy to containerize.**  
We are dedicated to empowering game server administrators by seamlessly integrating essential tooling—such as mod installations, Discord notifications, and more—directly into the container. Our goal is to simplify server deployment, management, and scalability while fostering a vibrant, community-driven ecosystem.

## Catalog

[ready]: https://img.shields.io/badge/Status-ready-green?style=for-the-badge
[development]: https://img.shields.io/badge/Status-development-orange?style=for-the-badge

- **Ready** - The game server manager is ready for use.
- **Development** - The game server manager is in development.
- **Planned** - The game server manager is planned but not yet started.
- **Origin** - This repository is the origin of the game server manager.

| Game       | Repository                                                          | Status         |
| ---------- | ------------------------------------------------------------------- | -------------- |
| Valheim    | [Valheim Docker](https://github.com/mbround18/valheim-docker)       | Origin         |
| Palworld   | [Palworld Docker](https://github.com/mbround18/palworld-docker)     | ![ready]       |
| Enshrouded | [Enshrouded Docker](https://github.com/mbround18/enshrouded-docker) | ![development] |
| gsm-cli    | This repository                                                     | ![development] |

## Repository Structure

- **libs/** - A library of reusable code for building game server managers.
- **apps/** - CLI tools for managing game servers.
  - Each game-specific folder includes its own README with usage instructions and repository links.
  - `gsm-cli` is the generic SteamCMD-based lifecycle wrapper for install, start, stop, restart, update, and generic monitoring.

## Build & Usage

This project uses a `Makefile` for streamlined development. The primary commands include:

```sh
make lint         # Format and lint the Rust code
make test         # Run tests
make build        # Build the project
make docker-build # Build the Docker container
make docker-push  # Push the built container to the registry
```

### gsm-cli

`gsm-cli` is a generic Steam dedicated server manager built on top of the shared `gsm-instance` crate. It is intended for cases where you know the Steam App ID, install path, executable, and compatibility mode you want to run, but do not need game-specific configuration bootstrapping.

Supported commands:

```sh
gsm-cli install --app-id 2394010 --install-path /home/steam/palworld
gsm-cli start --app-id 2394010 --install-path /home/steam/palworld --executable /bin/bash --launch-arg ./PalServer.sh
gsm-cli stop --app-id 2394010 --install-path /home/steam/palworld --executable /bin/bash
gsm-cli restart --app-id 2278520 --install-path /home/steam/enshrouded --executable enshrouded_server.exe --force-windows
gsm-cli update --app-id 2394010 --install-path /home/steam/palworld
gsm-cli update --app-id 2394010 --install-path /home/steam/palworld --check
gsm-cli monitor --app-id 2394010 --install-path /home/steam/palworld --update-job
```

Environment fallbacks are supported for the same runtime contract:

- `APP_ID`
- `INSTALL_PATH`
- `EXECUTABLE` or `COMMAND`
- `LAUNCH_MODE` with `native`, `wine`, or `proton`
- `FORCE_WINDOWS`
- `INSTALL_ARGS`
- `LAUNCH_ARGS`

CLI flags take precedence over environment variables. For runtime commands such as `start`, `stop`, and `restart`, the executable is required because `gsm-cli` does not persist game profiles.

Generic scope intentionally excludes:

- automatic executable discovery
- built-in game profiles
- game-specific config file generation
- game-specific webhook or log parsing rules

### Docker

The workspace includes `apps/*` automatically, so `apps/gsm-cli` is built with the rest of the project. The final stage in [Dockerfile](Dockerfile) copies release binaries into `/usr/local/bin`, which means `gsm-cli` is shipped alongside the other compiled CLI tools when you build the final image target.

`compose.yml` now includes a `windrose` service that uses the generic `gsm-cli` contract on top of the Proton-capable final image. Because Windrose is Windows-only and the executable name could not be resolved reliably from official metadata in this environment, the service requires you to provide `WINDROSE_EXECUTABLE` before startup. Example:

```sh
WINDROSE_EXECUTABLE=YourWindroseServer.exe docker-compose up --build windrose
```

The service sets `APP_ID=4129620`, `FORCE_WINDOWS=true`, and `LAUNCH_MODE=proton`, and persists both the installed server files and Proton compat data under `./data/windrose/`.

## Contributing

Contributions are welcome! To contribute:

1. Fork the repository.
2. Create a feature branch (`git checkout -b feature-branch`).
3. Commit your changes (`git commit -m 'Add new feature'`).
4. Push to your branch (`git push origin feature-branch`).
5. Open a pull request.

Ensure your contributions adhere to our coding style and include appropriate tests where applicable.

## License

This project is licensed under the BSD 3-Clause License. Portions of the software are derived from [mbround18/valheim-docker](https://github.com/mbround18/valheim-docker). See the [LICENSE](LICENSE) file for details.

## Contact & Support

For questions or support, please open an issue in this repository.

## Acknowledgments

Thank you to the contributors of [mbround18/valheim-docker](https://github.com/mbround18/valheim-docker) whose work has significantly influenced this project.
