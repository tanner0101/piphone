# Pi Can Phone

Modern day tin can phone.

## Getting Started

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
