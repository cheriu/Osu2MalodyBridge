# osu2malody-store

Allow [Malody V](https://mugzone.net/) client to browse, search, and download osu!mania beatmaps directly.

Built with Rust + [rosu-v2](https://github.com/MaxOhn/rosu-v2).

[中文文档](README_CN.md)

## Acknowledgments

This project is inspired by [flben233/OsuToMalodyServer](https://github.com/flben233/OsuToMalodyServer). Thanks to the original author for laying the groundwork for osu! → Malody bridging.

## Content

- [Usage](#usage)
- [Configuration](#configuration)
- [Download](#download)
- [Build from Source](#build-from-source)
- [Contribute](#contribute)
- [License](#license)

## Usage

0. Create an OAuth application at [osu! account settings](https://osu.ppy.sh/home/account/edit#new-oauth-application) (callback URL can be blank).

1. Download the latest release for your platform from [Releases](https://github.com/cheriu/Osu2MalodyBridge/releases).

2. Unzip and edit `application.yml` with your osu! API credentials:

   ```yaml
   malody:
     osu:
       clientID: 12345
       clientSecret: your-osu-oauth-secret
   ```

   Or use environment variables instead:
   ```bash
   export OSU_CLIENT_ID=12345
   export OSU_CLIENT_SECRET=your-osu-oauth-secret
   ```

3. Run the server:
   - **Linux**: `./osu2malody-store`
   - **Windows**: `osu2malody-store.exe`

4. In Malody V, go to **Settings → Store → Custom Server**, enter `http://your-server-ip:8081`.

5. Enjoy!

## Configuration

All settings in `application.yml`, overridable via environment variables.

```yaml
server:
  port: 8081                        # listen port
  # bind_address: "::"              # bind address, defaults to dual-stack
  # tls_cert: /path/to/cert.pem     # TLS certificate
  # tls_key: /path/to/key.pem       # TLS private key

malody:
  server:
    api: 202310                     # server API version
    min: 202310                     # min client API version
    welcome: "Welcome!"             # welcome message
    tmp: /tmp/Osu2Malody            # .osz cache directory
    # verify_client_auth: false     # verify client uid/key signature
  osu:
    clientID:                       # osu! OAuth Client ID
    clientSecret:                   # osu! OAuth Client Secret
```

### HTTPS

Set both `tls_cert` and `tls_key` to enable HTTPS. When behind a reverse proxy, keep HTTP and set the `X-Forwarded-Proto` header.

### Client Auth

When `verify_client_auth: true`, the server verifies the RSA signature on client `uid`/`key`/`api` parameters, rejecting forged or tampered requests.

## Download

[Releases](https://github.com/cheriu/Osu2MalodyBridge/releases)

Pre-built packages for Linux (x86_64) and Windows (x86_64).

## Build from Source

```bash
git clone https://github.com/cheriu/Osu2MalodyBridge.git
cd Osu2MalodyBridge/o2m-rust
cargo build --release
# binary at: target/release/osu2malody-store
```

## Contribute

Welcome to open an [Issue](https://github.com/cheriu/Osu2MalodyBridge/issues/new) or Pull Request.

## License

[MIT](LICENSE)
