# REFresher

![alt text](zombie.svg "Title")

### REFramework updater CLI app

REFresher is a simple CLI app that aims
to update new [nightly](https://github.com/praydog/REFramework-nightly/releases) releases of [Praydog](https://www.patreon.com/praydog) - [REFramework](https://github.com/praydog/REFramework) VR mod.

## Features

- After first run app create ms-link for each supported game. By executing the link app will check if new REFramework mod update is available for that game is so download it, unpack and launch the game. All ms-links are located in REFR_link folder.
- Load older version of REFramework mod from cache (default it will cache 4 mod versions per game)
- Switch between Nextgen/Standard mod versions for RE2, RE3, RE7 games.
- Switch between OpenXR/OpenVR mod versions for all games.
- At first run app will Steam detect all supported games and current REFramework mod settings per game if mod is installed.

## Build

requirements - https://www.rust-lang.org/tools/install

System specific:

### Windows

App uses icon.ico for its exe. It handles this by winres crate - https://github.com/mxre/winres in order to have exe build with icon follow instructions on winres crate github. If not delete build.rs before build.

```sh
cargo build --target x86_64-pc-windows-msvc # dev build
cargo build --release --target x86_64-pc-windows-msvc # release build
```

You might need to set your specific location of rc.exe file folder by changing

```rs
res.set_toolkit_path(r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64");
```

to your Windows Kits bin folder location

### Linux/WSL

I don't know how to build it with icon on linux. But without you can use.

```sh
cross build --target x86_64-pc-windows-gnu # dev build
cross build --release --target x86_64-pc-windows-gnu # release build
```

It might be required to remove build.rs file before building.

## Testing

requirements:

```sh
cargo install cargo-nextest --locked
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```

### Windows

```sh
cargo nextest run --target x86_64-pc-windows-msvc
```

to generate html report

```sh
cargo llvm-cov nextest --target x86_64-pc-windows-msvc
cargo llvm-cov nextest  --html --target x86_64-pc-windows-msvc # html report -> output target\llvm-cov\html
```

### Linux

```sh
cross nextest run --target x86_64-pc-windows-gnu
cross llvm-cov --target x86_64-pc-windows-gnu # not sure if it works
```
