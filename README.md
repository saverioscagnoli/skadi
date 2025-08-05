# Skadi 🧊

Skadi is a web-powered widget system, it is powered by webkit6 and gtk, making it extremely customizable by using web techologies and frameworks like React, Svelte, Vue, etc.

## If you're concerned about resources

Keep in mind that since it is powered by webkit6, this will be pretty memory intensive, and it WILL take a lot of ram. (on my system: arch x86_64 32gb takes up about 200MB of ram), So if that's a concern, please consider something else like [eww](https://github.com/elkowar/eww).

I think that the tradeoff between resource usage and customizability is worth, that's why I made this.

## Installation

To install skadi, if you can just grab the precompiled binary for x86_64 in the releases. Otherwise you can compile it yourself, but it requires cargo and rust.

To compile it, just clone the repo
and put the binary in $PATH.

```
git clone https://github.com/saverioscagnoli/skadi.git
cd skadi
cargo build --release
sudo cp ./target/release/skadi /usr/bin
```

## Usage

This is basically an automated vite project managed by a cli and ad webkit6 app to display them.

### Cli args

- `--skip-requirements`: Skips checking for requirements like nodejs and npm
- `--skip-vite`: Skips building the vite project, this will save up a lot of time, but use it only if you've already built it previously.
- `--dev`: Spins up the vite dev server, so you can edit plugins and it will display the changes immediately!
- `--debug`: Enables debug logging and web inspector.
- `--workspace-dir`: Path to the managed vite workspace. The default path is `~/.local/share/skadi`.
- `--show-ouput`: Shows the output of `vite build` and `vite dev` in the terminal.

## Dependencies

- [webkit](https://webkit.org/) (more specifically the rust crate [webkit6](https://docs.rs/webkit6/latest/webkit6/))
- [nodejs](https://nodejs.org)
- [npm](https://www.npmjs.com/)

Nodejs and npm will be automatically detected at the start of the program if not using `--skip-requirements`

## Examples

My personal topbar
![my-bar](./docs/screenshots/my-bar.png)

You can contribute some examples if you'd like!

## License

MIT License (c) 2025 Saverio Scagnoli
