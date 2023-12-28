# hyprprop-rust
xprop for Hyprland, a Rust port from https://github.com/vilari-mickopf/hyprprop

## Dependencies
- Only slurp besides what's in Cargo
- Slurp location can be changed with the SLURP_LOCATION environment variable, which is the whole point of the rewrite in the first place (to allow installing in NixOS)

## Install

#### Through bash:
```bash
export SLURP_LOCATION=/usr/bin/slurp
cargo install --path .
```
Installed binary is at ~/.cargo/bin

--------------------------------
#### Alternatively, with Flakes: (currently only supported system is x86_64-linux)

flake.nix:
```nix
inputs {
    hyprprop-rust = {
      url = "github:PacificViking/hyprprop-rust";
      inputs.nixpkgs.follows = "nixpkgs";
    };
};
```
configuration.nix:
```nix
environment.systemPackages = [
    inputs.hyprprop-rust.defaultPackage.${settings.systemtype}
]  # don't use "with pkgs" here because its under inputs

```

## Todo
- There is an async loop that returns when should_reload is toggled: this isn't too elegant
- Whenever you switch to an empty workspace, the program aborts immediately.
