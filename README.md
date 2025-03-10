# Game Server Management

Game Server Management is the culmination of everything I learned from [mbround18/valheim-docker](https://github.com/mbround18/valheim-docker) and its Rust code.

The purpose of this repository is to provide a Docker container containing CLI tools designed for building game-specific containers.

## Repository Structure

- **libs/** - A library of reusable code for building game server managers.
- **apps/** - CLI tools for managing game servers.
  - These tools are not intended to be used directly; use at your own discretion.
  - Each game under this folder will have its own README with usage instructions and a link to the repository where it is consumed.

## Build & Usage

### Makefile

This project is managed using `make`. Below are the primary commands:

```sh
make lint         # Format and lint the Rust code
make test         # Run tests
make build        # Build the project
make docker-build # Build the Docker container
make docker-push  # Push the built container to the registry
```

## License

This project is licensed under the BSD 3-Clause License. Portions of this software are derived from the repository [mbround18/valheim-docker](https://github.com/mbround18/valheim-docker). See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! If you'd like to contribute:

1. Fork the repository.
2. Create a feature branch (`git checkout -b feature-branch`).
3. Commit your changes (`git commit -m 'Add new feature'`).
4. Push to your branch (`git push origin feature-branch`).
5. Open a pull request.

All contributions must adhere to the existing coding style and include appropriate tests where applicable.

## Contact & Support

If you have questions, open an issue in this repository.

---

### Acknowledgments

Thanks to all contributors of [mbround18/valheim-docker](https://github.com/mbround18/valheim-docker) for their prior work, which has influenced this project.
