# Game Server Management

Game Server Management is the culmination of lessons learned from [mbround18/valheim-docker](https://github.com/mbround18/valheim-docker) and its robust Rust code. This repository provides a Docker container enriched with CLI tools for crafting game-specific containers.

## Manifesto

Our mission is clear: **All SteamCMD dedicated servers should be easy to containerize.**  
We are dedicated to empowering game server administrators by seamlessly integrating essential tooling—such as mod installations, Discord notifications, and more—directly into the container. Our goal is to simplify server deployment, management, and scalability while fostering a vibrant, community-driven ecosystem.

## Catalog

[ready]: https://img.shields.io/badge/Status-ready-green?style=for-the-badge
[development]: https://img.shields.io/badge/Status-development-orange?style=for-the-badge
[planned]: https://img.shields.io/badge/Status-ready-yellow?style=for-the-badge

- **Ready** - The game server manager is ready for use.
- **Development** - The game server manager is in development.
- **Planned** - The game server manager is planned but not yet started.
- **Origin** - This repository is the origin of the game server manager.

| Game       | Repository                                                          | Status         |
| ---------- | ------------------------------------------------------------------- | -------------- |
| Valheim    | [Valheim Docker](https://github.com/mbround18/valheim-docker)       | Origin         |
| Palworld   | [Palworld Docker](https://github.com/mbround18/palworld-docker)     | ![development] |
| Enshrouded | [Enshrouded Docker](https://github.com/mbround18/enshrouded-docker) | ![development] |

## Repository Structure

- **libs/** - A library of reusable code for building game server managers.
- **apps/** - CLI tools for managing game servers.
  - Each game-specific folder includes its own README with usage instructions and repository links.

## Build & Usage

This project uses a `Makefile` for streamlined development. The primary commands include:

```sh
make lint         # Format and lint the Rust code
make test         # Run tests
make build        # Build the project
make docker-build # Build the Docker container
make docker-push  # Push the built container to the registry
```

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

```

```
