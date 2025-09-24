# 🎧 Audiothek Quickplay

`audiothek-quickplay` is a simple, fast command-line tool built in Rust to instantly play the latest episode of any podcast from the German ARD Audiothek.

It provides a straightforward way to stay up-to-date with your favorite shows without needing a browser or a graphical application. 🚀

## ✨ Features

- **Instant Playback**: Launch and play the latest episode of a pre-configured podcast with a single command.
- **Interactive Menu**: If no podcast is specified, an interactive menu allows you to choose from your list.
- **Cross-Platform**: Compiles and runs on both Linux 🐧 and Windows 🪟.
- **Interactive Player**: In-terminal controls for play/pause and seeking.

## 📥 Downloads

Get the latest pre-compiled binaries from the [**GitHub Releases**](https://github.com/Fensterbank/audiothek-quickplay/releases/latest) page.

- [**Windows (x86_64)**](https://github.com/Fensterbank/audiothek-quickplay/releases/latest/download/audiothek-quickplay-windows-x86_64.zip)
- [**Linux (x86_64)**](https://github.com/Fensterbank/audiothek-quickplay/releases/latest/download/audiothek-quickplay-linux-x86_64.tar.gz)

## ⚙️ Configuration

The application requires a `podcasts.json` file in the same directory as the executable. This file defines the podcasts you want to access.

1.  **Find the Show ID**: Go to the ARD Audiothek website and find the page for your desired podcast. The URL will contain the ID. For example, for `https://www.ardaudiothek.de/sendung/die-nachrichten/urn:ard:show:8ca6599d43089072/`, the ID is `urn:ard:show:8ca6599d43089072`.

2.  **Edit `podcasts.json`**: Add an entry for each podcast with a unique `key` (a short name you'll use in the command line) and the `id` you found.

    ```json
    [
      {
        "key": "nachrichten",
        "id": "urn:ard:show:8ca6599d43089072"
      },
      {
        "key": "infodate",
        "id": "urn:ard:show:15f007e3b43a4a8c"
      },
      {
        "key": "15minuten",
        "id": "urn:ard:show:b84b465ae5abcd64"
      },
      {
        "key": "diewoche",
        "id": "urn:ard:show:c570a55d7067784d"
      }
    ]
    ```

## 🚀 Usage

Place the compiled binary and the `podcasts.json` file in the same directory. You can then run the application from your terminal.

**Run with interactive selection:**

```bash
./audiothek_quickplay
```

**Play a specific podcast directly:**

```bash
./audiothek_quickplay <key>
```

*Example:*

```bash
./audiothek_quickplay infodate
```

### Player Controls

- **[Space]**: Pause / Play
- **[<-]**: Rewind 10 seconds
- **[->]**: Forward 10 seconds
- **[q]** or **[Esc]**: Quit the player

## 🛠️ Building from Source

First, ensure you have the Rust toolchain installed. The `podcasts.json` file will be automatically copied to the output directory during the build process.

### For Linux

```bash
cargo build --release
```

The binary will be located at `target/release/audiothek_quickplay`.

### For Windows (Cross-compilation from Linux)

Make sure you have the `x86_64-pc-windows-gnu` target installed:

```bash
rustup target add x86_64-pc-windows-gnu
```

Then, build the project:

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

The binary will be at `target/x86_64-pc-windows-gnu/release/audiothek_quickplay.exe`.

## 🚀 Releasing a New Version

This project uses GitHub Actions to automate the release process. To publish a new version, follow these steps:

1.  **Update Version Number**: Change the `version` in `Cargo.toml` to the new version (e.g., `0.1.1`).

2.  **Commit and push your changes**.

3.  **Tag the Release**: Create a new Git tag that matches the version number. The tag **must** start with a `v`.
    ```bash
    git tag v0.1.1
    ```

4.  **Push the Tag**: Push the new tag to GitHub. This will trigger the release workflow.
    ```bash
    git push origin v0.1.1
    ```

GitHub Actions will then automatically build the binaries, create a new release, and upload the packaged files.

## 📄 Author & License
MIT License. Built with ❤️ by Frédéric Bolvin, [f-bit software](https://f-bit.software).  
