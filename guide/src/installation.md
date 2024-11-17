# Installation

## HW requirements

- at least 1 CPU core

- at least 1 GB memory

- at least 1 GB disk space

## Install steps

- Install dependencies, e.g. on Fedora:

```bash
dnf install cargo
dnf install clang-devel
dnf install git
dnf install libicu-devel
dnf install llvm-devel
dnf install make
dnf install npm
dnf install openssl-devel
dnf install sqlite-devel
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
osm-gimmisn sync-ref --mode download --url https://osm.example.com/data/
```

## Install steps (Windows)

- Install [git](https://git-scm.com/download/win) and
  [cargo](https://www.rust-lang.org/tools/install).

- Open e.g. PowerShell and clone the repo (see above).

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

`workdir/wsgi.ini` contains the configuration. Common keys to be customized (showing the defaults):

```toml
uri_prefix = '/osm'
tcp_port = '8000'
overpass_uri = 'https://z.overpass-api.de'
cron_update_inactive = 'False'
```

## Running within a container

You can try osm-gimmisn in 5 minutes following these basic steps:

1. Clone the repo:

```
git clone https://github.com/vmiklos/osm-gimmisn
```

2. Build the container:

```
cd osm-gimmisn/tools/container && ./build.sh
```

3. Run the container:

```
./run.sh
```

4. Sync the reference data:

```
podman exec -t -i osm-gimmisn bash -c 'cd /opt/osm-gimmisn && target/release/osm-gimmisn sync-ref --mode download --url https://osm.example.com/data/'
```

5. Go to <http://0.0.0.0:8000/osm/> in your web browser.

Note that the comparison results are not only affected by changes in the OSM database, false
positives are also silenced by osm-gimmisn filters. To update those filters, run:

```
podman exec -t -i osm-gimmisn bash -c 'cd /opt/osm-gimmisn && git pull -r && target/release/osm-gimmisn cache-yamls data workdir'
```

from time to time.

Also note that you can decide what container manager you want to use. It's not a problem if you
prefer `docker` instead of `podman`, the commands documented here are still meant to work.
