# DynamicArena
This is a 2D Fighting Game developed for the school festival of my high school.
This game has two mode to play, Single and Multi. In Single mode, you will fight against the rule-base bot. And in Multi mode(this is only available on-site), you will fight with your buddy.

You can preview the current progress by visiting [PC edition](https://mommawatasu.github.io/DynamicArena/) or [mobile edition](https://mommawatasu.github.io/DynamicArena/phone).

## Build
To build this project, you should use cargo-make which can be installed with following command.
```
$ cargo install --force cargo-make
```
Then, you can use these tasks:
- BUILD: just build the project using dynamic linking feature
- DEV: build with dev profile and run on native environment using dynamic linking feature
- wasmDEV: build with dev profile for wasm target and launch the server which listens on 1334 port using wasm server runner
- RELEASE: build with release profile(heavier than dev profile) and run on native environment
- wasmRELEASE: build with release profile(slightly different from above, since this option tries to shrink the size of binary)
- wasm-phoneRELEASE: almost the same as wasmRELEASE, but for the mobile edition
- wasmDEPLOY: move files made by wasmRELEASE task into the docs directory and launch the server which listens on 8000 port
- wasm-phoneDEPLOY: almost the same as wasmDEPLOY, but for the mobile edition

In order to enable the access from the other devices into the local server on wasmDEV task. You have to edit(or create) `.cargo/config.toml` as follows:
```
[env]
WASM_SERVER_RUNNER_ADDRESS = "0.0.0.0"

[target.wasm32-unknown-unknown]
runner = "wasm-server-runner"
```
