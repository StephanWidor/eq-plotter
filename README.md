# eq-plotter

Playing around to learn some Rust:

- using egui for plotting impulse and frequency response, as well as poles and zeros of biquad eqs.
- using nih-plug for building an eq audio plugin

https://github.com/user-attachments/assets/aac8b0cc-324e-41b8-9f55-36b7f83f08f3

## Building:

### egui app:

#### native:
```
cargo build -p eq-plotter-egui
```
Find the binary in the target folder, or run by
```
cargo run --bin eq-plotter-egui
```

#### wasm:
```
cargo build -p eq-plotter-egui --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/eq-plotter-egui.wasm --out-dir ./wasm_out --web
python3 -m http.server --directory ./wasm_out 8080
```
Then you can run eq-plotter in your browser under localhost:8080.

### slint app:
```
cargo build -p eq-plotter-slint
```
Find the binary in the target folder, or run by
```
cargo run --bin eq-plotter-slint
```

#### Remark:
Drawing plots in slint is done via plotters crate. This seems to be very slow in debug builds (or at least I didn't find a way to speed that up).
Building release builds is done by adding `--release` to `cargo build`/`cargo run`.

### nih plugin:

#### Standalone:
```
cargo build -p eq-plugin-egui
```
Find the binary in the target folder, or run by
```
cargo run --bin eq-plugin-egui
```
#### Audio plugin:
```
cargo xtask bundle eq-plugin-egui --release
```
VST3 and Clap plugin can then be found in target/bundled
