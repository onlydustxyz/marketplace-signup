<p align="center">
    <img src="resources/img/logo.png">
</p>
<div align="center">
  <h1 align="center">Badge Signup</h1>
  <p align="center">
    <a href="https://discord.gg/onlydust">
        <img src="https://img.shields.io/badge/Discord-6666FF?style=for-the-badge&logo=discord&logoColor=white">
    </a>
    <a href="https://twitter.com/intent/follow?screen_name=onlydust_xyz">
        <img src="https://img.shields.io/badge/Twitter-1DA1F2?style=for-the-badge&logo=twitter&logoColor=white">
    </a>
    <a href="https://contributions.onlydust.xyz/">
        <img src="https://img.shields.io/badge/Contribute-6A1B9A?style=for-the-badge&logo=notion&logoColor=white">
    </a>
  </p>
  
  <h3 align="center">Handles GitHub users signup / badge creation</h3>
</div>

> ## ⚠️ WARNING! ⚠️
>
> This repo contains highly experimental code.
> Expect rapid iteration.

## 🎟️ Description

This backend application handles user signup (ie. badge creation) for GitHub users.

## 🎗️ Prerequisites

Rust installed.

## 📦 Installation

## 🔬 Usage

### Configuration

All these environment variable must be set with appropriate values:

- `GITHUB_ID` The GitHub OAuth App client ID
- `GITHUB_SECRET` The GitHub OAuth App client secret
- `STARKNET_ACCOUNT` Badge-Registry's owner account contract address
- `STARKNET_PRIVATE_KEY` Badge-Registry's owner private key
- `STARKNET_BADGE_REGISTRY_ADDRESS` Badge-Registry contract address
- `STARKNET_CHAIN` Either MAINNET or TESTNET

Optional:

- `ROCKET_LOG_LEVEL` Max level to log. (off/normal/debug/critical). Default for release: critical.

### Run locally (dev)

```bash
cargo run
```

# Open Telemetry

```bash
docker-compose -f resources/docker-compose/dev/docker-compose.yml up -d
```

Access to Jaeger UI: 
http://localhost:16686/

### Build

```bash
cargo build --release
```

### Run executable

```bash
./target/release/od-badge-signup
```

## 🌡️ Testing

```bash
cargo test
```
