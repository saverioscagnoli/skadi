# Development Guide

## Understanding the Build System

`wwwidgets` has two different modes of operation that affect how your widgets are served:

### Production Mode (Default)
```bash
cargo run --release
# or
wwwidgets
```

**How it works:**
1. Reads config from `~/.config/wwwidgets/config.json`
2. Generates widget entry points in `~/.local/share/wwwidgets/jsx/` and `~/.local/share/wwwidgets/html/`
3. Runs `yarn build` to create optimized production bundles
4. Outputs built files to `~/.local/share/wwwidgets/build/`
5. Starts a server on `localhost:10978` serving the **cached build**
6. WebView loads from `http://localhost:10978/html/{widget}.html`

**⚠️ Important:** The build is cached! If you make changes to your config or widget code, you need to either:
- Run `wwwidgets clean` to clear the cache
- Use development mode instead

### Development Mode (Recommended for Development)
```bash
cargo run --release -- --dev
# or
wwwidgets --dev
# or use the helper script
./dev.sh run
```

**How it works:**
1. Reads config from `~/.config/wwwidgets/config.json`
2. Generates widget entry points in `~/.local/share/wwwidgets/jsx/` and `~/.local/share/wwwidgets/html/`
3. Starts Vite dev server on `localhost:5173` with hot module replacement
4. WebView loads from `http://localhost:5173/html/{widget}.html`

**✅ Benefits:**
- Hot reload: Changes to your widgets are reflected immediately
- No build cache issues
- Better error messages
- Faster iteration

## Common Issues

### "I deleted a config but it's still showing the old version!"

This happens when you're running in **production mode** without clearing the cache. The old build is still in `~/.local/share/wwwidgets/build/`.

**Solutions:**
1. Clear the cache:
   ```bash
   wwwidgets clean
   # or
   ./dev.sh clean-cache
   # or manually
   rm -rf ~/.local/share/wwwidgets/build/
   ```

2. Use development mode instead:
   ```bash
   wwwidgets --dev
   ```

### "Changes to my widget aren't showing up!"

**If using production mode:**
- The build is cached. Clear it with `wwwidgets clean` or use `--dev` mode

**If using development mode:**
- Check the Vite dev server output for errors
- Make sure your file is being watched by Vite
- Check the browser console (use `--debug` flag to enable inspector)

### "How do I debug my widgets?"

Enable the WebView inspector:
```bash
wwwidgets --debug
# or in dev mode
wwwidgets --dev  # dev mode automatically enables debug
```

Then you can right-click on your widget and select "Inspect Element" (if supported by your compositor).

## Directory Structure

```
~/.config/wwwidgets/
├── config.json              # Your main configuration
├── use-backend.ts           # Generated backend hooks
└── your-widgets/            # Your widget source files
    └── sidebar.tsx

~/.local/share/wwwidgets/
├── build/                   # Production build output (CACHED!)
│   ├── assets/              # Bundled JS/CSS
│   └── html/                # HTML entry points
├── jsx/                     # Generated JSX entry points
├── html/                    # Generated HTML templates (for Vite)
├── package.json             # Generated package.json
├── vite.config.ts           # Generated Vite config
└── node_modules/            # Node dependencies
```

## Development Workflow

### Recommended Workflow

1. **During active development:**
   ```bash
   ./dev.sh run
   ```
   Use dev mode for hot reload and instant feedback.

2. **Testing production build:**
   ```bash
   ./dev.sh clean-cache
   cargo run --release
   ```
   Clear cache and test the production build.

3. **Installing for daily use:**
   ```bash
   ./dev.sh install
   ```
   Build and install to `~/.local/bin`.

### Helper Script Commands

The `dev.sh` script provides convenient shortcuts:

```bash
./dev.sh run          # Run in dev mode with hot reload
./dev.sh build        # Build release binary
./dev.sh clean        # Clean everything (Cargo + cache)
./dev.sh clean-cache  # Clean only build cache
./dev.sh install      # Build and install to ~/.local/bin
./dev.sh config       # Open config file in $EDITOR
./dev.sh logs         # Show build directory contents
./dev.sh help         # Show help message
```

## Debugging Tips

### Check what's being built

```bash
# See generated entry points
cat ~/.local/share/wwwidgets/jsx/sidebar.jsx
cat ~/.local/share/wwwidgets/html/sidebar.html

# See what's in the production build
ls -lah ~/.local/share/wwwidgets/build/
```

### View current config

```bash
cat ~/.config/wwwidgets/config.json
```

### Monitor Vite dev server

In `--dev` mode, Vite output is shown with `=>` prefix. Watch for errors or warnings.

### Use the inspector

Run with `--debug` or `--dev` to enable the WebView inspector and view console logs, network requests, and inspect the DOM.

## Building for Release

```bash
cargo build --release
# Binary will be at: target/release/wwwidgets
```

## FAQ

**Q: Why does dev mode work but production mode doesn't?**  
A: Your production build is cached. Clear it with `wwwidgets clean` or `./dev.sh clean-cache`.

**Q: How do I force a rebuild?**  
A: Either clear the cache or delete `~/.local/share/wwwidgets/build/`.

**Q: Can I use both modes at the same time?**  
A: No, they use different ports and serve different content. Pick one.

**Q: Where are my widget files?**  
A: Your widget source files should be in `~/.config/wwwidgets/` or wherever you specified in the `index` field of your config.

**Q: How do I share my setup?**  
A: Share your `~/.config/wwwidgets/config.json` and your widget source files. The build system will regenerate everything else.

## Contributing

When submitting PRs, please:
1. Test both production and development modes
2. Ensure `cargo clippy` passes
3. Test cache clearing functionality
4. Document any new configuration options

---

**Pro Tip:** During development, keep `./dev.sh run` running in a terminal and edit your widget files. Changes will hot-reload automatically! 🚀