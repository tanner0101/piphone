# piPhone

Modern day tin can phone built using Orange Pi 3B.

## Getting Started

Install Orange Pi OS (Arch) from [here](http://www.orangepi.org/html/hardWare/computerAndMicrocontrollers/service-and-support/Orange-Pi-3B.html) to an SD card using [balenaEtcher](https://etcher.balena.io/).

Get SSH and WiFi setup. You can use `nmcli` to connect to WiFi.

```
nmcli dev wifi connect <wifi_name> password <wifi_passwd>
```

Give the Pi a static DHCP assignment in your router and setup port forwarding for port `5060` (voip).

Install rust.

```
sudo pacman -S rustup
rustup default stable
```

Make sure your user is in `audio` group.

```
sudo usermod -aG audio $USER
```

You should see devices listed with the following command (no sudo).

```
alsactl info
```

Finally, run the app using cargo.

```
cargo run
```
