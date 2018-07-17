# edit-text

![Preview Image](https://user-images.githubusercontent.com/80639/42796598-12e44326-8959-11e8-8409-7a887ada2afa.png)

edit-text is a collaborative rich text editor, written in Rust with a frontend in WebAssembly.

👶 This is very early software.

📚 Read more in the [**Book on edit-text Internals**](http://tcr.github.io/edit-text/).

✏️ Or [**try out the edit-text sandbox**](http://sandbox.edit.io/).

## Getting Started

You'll need `cargo-script` to run the build script and a few other build tools:

```
cargo install cargo-script
cargo install wasm-bindgen-cli
cargo install diesel_cli --no-default-features --features sqlite-bundled
```

Install the WebAssembly target:

```
rustup target add wasm32-unknown-unknown
```

To test out the text editor live, run the server using this command. On first startup, this will first compile the database and wasm bundle. After that, this command launches the HTTP and WebSocket servers:

```
./x.rs wasm-build
./x.rs frontend-build
./x.rs server
```

Navigate to <localhost:8000> and start editing!

You can run the server in release mode easily with the `--release` flag:

```
./x.rs server --release
```

For more commands, run `./x.rs help`.

## Local client proxy in Rust (no WASM)

Start the sync server in one terminal with this switch:

```
./x.rs server --client-proxy --release
```

In another terminal, run the client proxy:

```
./x.rs client-proxy --release
```

You will see any failures appear in the client-proxy code that would appear in the browser console when in WASM mode.

### Compiling the frontend

The bundled frontend code (written in TypeScript) is tracked in git, but you can also compile it yourself. Make sure you have Node installed first, then:

```
npm i --prefix ./edit-frontend
./x.rs frontend-watch
```

This command watches the `edit-frontend` directory and continuously builds all frontend code. Note that the .wasm bundle isn't inlined into the bundle with webpack, but loaded asynchronously.

## Deploying

[macOS Instructions] To deploy to a Dokku instance, you'll have to install a linux cross-compiler. This will cross-compile the Linux binary locally and then package it in a Docker container for distribution.

First, to install the cross compiler:

```
brew install qinyao-he/homebrew-gcc_cross_compilers/x64-elf-gcc
```

Then add a new Rust target:

```
rustup target add x86_64-unknown-linux-gnu
```

Then modify `./x.rs` (for the moment) to add your hostname and credentials, then:

```
./x.rs deploy
```

## License

Apache-2.0
