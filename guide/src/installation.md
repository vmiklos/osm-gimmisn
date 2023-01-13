# Installation

## HW requirements

- at least 1 CPU core

- at least 1 GB memory

- at least 1 GB disk space

## Install steps

- Install dependencies, e.g. on Fedora:

```bash
dnf install git
dnf install make
dnf install npm
dnf install cargo
dnf install openssl-devel
dnf install libicu-devel
dnf install llvm-devel
dnf install clang-devel
```

If the `npm` version in your distribution is too old: `sudo npm install -g n && sudo n stable`.

- Clone the repo:

```bash
git clone https://github.com/vmiklos/osm-gimmisn
cd osm-gimmisn
```

- Build the code and cached data:

```bash
make
```

Populate the reference directory with TSV files for the house number and street list:

```bash
osm-gimmisn sync-ref --mode download --url https://www.example.com/osm/data/
```

## Install steps (Windows)

- Install `git` and `cargo`.

- Open e.g. the Command Prompt and clone the repo (see above).

- Build the code:

```bash
cargo build --release --no-default-features
```

- Run the validator:

```bash
osm-gimmisn.exe validator data\relation-budapest_11.yaml
```

## Developer setup

```bash
make run
```

This allows accessing your local instance for development.

## Production setup

- The launcher is `osm-gimmisn rouille`.

- A sample `osm-gimmisn.service` is provided, you can copy that to
  `/etc/systemd/system/osm-gimmisn.service` and customize it to your needs.

- Use `systemctl start osm-gimmisn.service` to run the app server.

- Optionally, set up a reverse proxy with SSL support.

- Optionally, add `cron` as a daily crontab:

```cron
# daily, at 0:05
5 0 * * * cd /home/osm-gimmisn/git/osm-gimmisn && target/release/osm-gimmisn cron --mode all
```

See `osm-gimmisn cron --help` for details on what switches are supported for that tool.

## Custom configuration

`wsgi.ini` contains the configuration. Common keys to be customized (showing the defaults):

```toml
uri_prefix = '/osm'
tcp_port = '8000'
overpass_uri = 'https://z.overpass-api.de'
cron_update_inactive = 'False'
```
