# Rust API Template

This repository serves as a **template for building APIs in Rust**. It provides a solid foundation with recommended structure and configuration files to help you get started quickly and maintain consistency across projects.

## Getting Started

To use this template for your own API, follow these steps:

### 1. Set Your API Name

You must define the name of your API in the following files:

- `Cargo.toml` – Update the `[package]` name to match your API.
- `docker-compose.yml` and/or `Dockerfile` – Set the appropriate service/image name.
- `nginx.conf` – Adjust upstream and server blocks to reflect your API name.

This ensures correct identification and deployment of your API.

### 2. Document Your API

Don't forget to fill out the `API.md` file to describe your API's endpoints, authentication mechanisms, error handling, and any other relevant documentation. This is crucial for both development and future maintenance.

---

Feel free to fork this template and customize it according to your project needs. Contributions and suggestions are welcome!

## Integration tests (newman)

The end-to-end scenario lives in `tests/postman/collection.json` (Postman v2.1 collection) with
its variables in `tests/postman/environment.json`. CI replays it with the `postman/newman` image
against the published `dev-<sha>` API image (`IMAGE_REF`), without any Postman account. Locally,
leave `IMAGE_REF` empty and the script builds `message-api:local` from `development.Dockerfile`:

```bash
./integration_test.sh
```

The script starts `docker-compose-integration.yml` (Postgres + Liquibase + seeder + Redis + API +
newman), waits for the `newman` service, prints its report and exits with its status (`--bail`
stops at the first failing request). To iterate on the collection against a stack already running
on `localhost:3003`:

```bash
docker run --rm --network host -v "$PWD/tests/postman:/etc/newman:ro" postman/newman:6.1.3-alpine \
  run collection.json --environment environment.json
```
