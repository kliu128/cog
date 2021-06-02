# cog

A dead simple user-level service manager for Linux written in Rust.

## Why?

Sometimes you want to just run a few programs in the background (e.g. [i3 exec](https://wiki.archlinux.org/title/i3#Autostart)) without writing some lengthy [systemd user services](https://wiki.archlinux.org/title/systemd/User).

## Usage

This repository builds two binaries: `cog` and `cogd`. When you first start `cogd`, it will generate an empty `.config/cogd.yml` file that you can populate with a list of processes to run. `cogd` will automatically listen for config file modifications and start/stop the required processes.

All services listed are expected to run forever; if they exit for any reason, `cogd` will wait 1 second and restart them.

To view the status of your processes, you can run `cog`, which will print out all processes known to `cogd` like so:

```
Services under management by cogd:
- [ RUNNING ] sleep infinity
...
```