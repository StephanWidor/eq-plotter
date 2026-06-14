# eq-plotter

Playing around to learn some Rust:

- using egui for plotting impulse and frequency response, as well as poles and zeros of biquad eqs.
- using nice-plug for building an eq audio plugin

https://github.com/user-attachments/assets/aac8b0cc-324e-41b8-9f55-36b7f83f08f3

## Building:

#### Standalone:
```
cargo build -p eq-plugin [--release]
```
Find the binary in the target folder, or run by
```
cargo run --bin eq-plugin [--release]
```
#### Audio plugin:
```
cargo xtask bundle eq-plugin [--release]
```
VST3 and Clap plugin can then be found in target/bundled
