# Clipboard manager for COSMIC™

![screenshot of the applet](https://media.githubusercontent.com/media/cosmic-utils/clipboard-manager/master/res/screenshots/main_popup.png)

The goal is to make a simple yet fast clipboard history, with a focus on UX, rapidity and security.

There is a quick settings popup when you right click the icon.

## Install

The applet is available in the cosmic-store.

You also build from source. For this you need to install [rust](https://rust-lang.org/tools/install/), [just](https://github.com/casey/just), and follow the the [build instruction](./BUILD.md).

## Logs

```sh
journalctl -p 3 -xb --user _EXE=/usr/bin/meowsystem-clipboard | less
```

- `-p` 3 means priority error
- `-x` add information
- `b` means since last boot

## Testing bundle

```sh
# install
flatpak install --user clipboard-manager.flatpak
# run specific branch
flatpak run --branch=testing io.github.meowsystem.Clipboard
# to be sure cosmic-panel will launch the wanted version
flatpak uninstall --user io.github.meowsystem.Clipboard//master
# or verify the commit with
flatpak run io.github.meowsystem.Clipboard -V
# uninstall testing repo and app
flatpak remote-delete --user meowsystem-clipboard-origin
```

## Build from source

Instructions are in [this file](./BUILD.md).

## Contributing

See [this file](./CONTRIBUTING.md).
