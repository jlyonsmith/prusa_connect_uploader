# Prusaconnect Uploader

[![coverage](https://shields.io/endpoint?url=https://raw.githubusercontent.com/jlyonsmith/prusaconnect_uploader/main/coverage.json)](https://github.com/jlyonsmith/prusaconnect_uploader/blob/main/coverage.json)
[![Crates.io](https://img.shields.io/crates/v/prusaconnect_uploader.svg)](https://crates.io/crates/prusaconnect_uploader)
[![Docs.rs](https://docs.rs/prusaconnect_uploader/badge.svg)](https://docs.rs/prusaconnect_uploader)

## Summary

Camera still uploader for Prusa Connect.  Loosely based on the [prusa-connect-camera-script](https://github.com/nvtkaszpir/prusa-connect-camera-script).

## Installation

Firstly, the script expects a single camera attached to the RPi via the ribbon cable (`/dev/video0`)  If you need to support multiple cameras, use the `prusa-connect-camera-script` or submit a pull request.

On your RPi, do the following:

```sh
sudo groupadd --system pi
sudo useradd --system --create-home --gid pi --groups audio,video,spi,i2c,gpio pi # Only video is strictly necessary
```

Then `sudo su pi` to become the `pi` user and `cd /home/pi`, then edit `/home/pi/.prusa-env` to contain:

```env
PRUSA_CONNECT_CAMERA_TOKEN=<your-token-here>
PRUSA_CONNECT_CAMERA_FINGERPRINT=<your-camera-fingerprint-here>
```

Generate a unique fingerprint for each camera using `uuidgen`.  You get the token when you add the camera in Prusa Connect.

Go back to being `root` then edit `/etc/systemd/system/prusa-connect-uploader@.service` to contain:

```conf
[Unit]
Description=Prusa Connect Camera Upload Service
After=network.target systemd-udev-trigger.service
Wants=network.target
Requires=systemd-udev-trigger.service

[Service]
User=pi
Group=pi
EnvironmentFile=/home/pi/.%i
ExecStart=/usr/local/bin/prusa-connect-uploader
Restart=always
RestartSec=15

[Install]
WantedBy=multi-user.target
```

Then run:

```bash
systemctl daemon-reload
systemctl enable prusa-connect-uploader@prusa-env.service
systemctl start prusa-connect-uploader@prusa-env.service
systemctl status prusa-connect-uploader@prusa-env.service
```

If all is well you will see the capture file, then the successful upload to Prusa Connect. Note that your printer must switched on an be connected to Prusa Connect for uploads to work.

If you are building from source, I recommend you use [Cross](https://github.com/cross-rs/cross) to cross compile for `--target aarch64-unknown-linux-gnu` if uploading to a Raspberry Pi. You can use the command `just cross user@host` to compile and upload in one step.
