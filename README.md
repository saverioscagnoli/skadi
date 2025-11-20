# wayland web widgets (wwwidgets)

A gtk and webkit based widget framework for wayland compositors.
All the powers of the web on your desktop!

## If you're skeptical
Keep in mind that it's powered by web technologies, it's memory hungry,
and it will take up a lot of memory.

I think that the trade off between resource usage and customization is worth it.

(On my system, Debian 13 x86_64, 32GB RAM, it takes up about 200MB)

## Installation
To install wwwidgets, you can compile it from source using `cargo`:

```bash
# Clone
git clone https://github.com/saverioscagnoli/wwwidgets
# Build
cd wwwidgets
cargo build --release
```

Then you can plug it in $PATH
```bash
sudo cp target/release/wwwidgets /usr/local/bin/
```

## wwwatch
wwwidgets also includes a utility called `wwwatch`, which is a simple
CLI tool useful for displaying information, such as workspaces, system information, notifications, and (soon) more.

## Usage
This is basically an automated Vite project, managed by the CLI tool to build and WebKit to display.

For backend communication, you have two options:

First, the `exec` function, which allows you to one-shot execute a command:
```js
// Relative to ~/.config/wwwidgets/
import { useBackend } from "./use-backend";

const { exec } = useBackend();
await exec("echo", ["Hello World!"]);
```

Second, the `useListen` hook (only for React), which allows you to spawn a long-running process, such as notification daemons, system information monitoring, etc.
```js
// Relative to ~/.config/wwwidgets/
import { useBackend } from "./use-backend";

const { useListen } = useBackend();

useListen("watch", ["ls", "-l"], (line) => {
  // `line` is the line of standard output from the process
  // ...
});
```

## Configuration
The configuration file must be located at `~/.config/wwwidgets/config.json`.
Here's where you define all of your windows.
Example: 

```json
{
  "widgets": [
    {
      "monitors": ["DP-1"],
      "label": "topbar",
      "width": "99.5%",
      "height": 35,
      "x": 0,
      "y": 5,
      "anchor": "top center",
      "exclusive": true,
      "index": "topbar/topbar.tsx",
      "background": [0, 0, 0],
      "opacity": 0.4
    },
    {
      "monitors": ["DP-1"],
      "label": "notifications",
      "width": 400,
      "height": 400,
      "x": 675,
      "y": 10,
      "anchor": "top right",
      "exclusive": false,
      "index": "topbar/components/notifications.tsx",
      "background": [0, 0, 0],
      "opacity": 0.4,
      "hidden": true
    }
  ]
}
```

## Arguments

- `debug`: Enables debug logging and web inspector for each webview.
- `dev`: Enables the development server, and enables hot reloading.
- `skip-requirements`: Skips the installation of requirements (blocks if they are not found [Node.js and npm]).

## Examples

![bar-1](docs/screenshots/bar-1.jpg)

## License
MIT License (c) 2025 Saverio Scagnoli
