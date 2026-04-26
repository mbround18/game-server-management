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

## Contributing

For information on how to contribute to this project, please see the [Contributing Guide](./docs/CONTRIBUTING.md).

## License

This project is licensed under the BSD 3-Clause License. Portions of the software are derived from [mbround18/valheim-docker](https://github.com/mbround18/valheim-docker). See the [LICENSE](LICENSE) file for details.

## Contact & Support

For questions or support, please open an issue in this repository.

## Acknowledgments

Thank you to the contributors of [mbround18/valheim-docker](https://github.com/mbround18/valheim-docker) whose work has significantly influenced this project.
