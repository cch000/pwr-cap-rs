# ryzencap

Flake that provides an easy way to tweak the power consumption of your Ryzen CPU.
It uses a daemon to look for unintended changes to the power values (e.g., changing power profile) 
and set them back to those specified in the config file.
Built using [ryzenadj libraries mappings to rust](https://crates.io/crates/libryzenadj).

Note: for information about supported CPUs check the 
[ryzenadj repo](https://github.com/FlyGoat/RyzenAdj).

## Usage

First, add it to your system flake inputs:

```nix
ryzencap = {
  url = "github:cch000/ryzencap";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

Then, you can use it by adding somewhere in your config:

```nix
imports = [
  inputs.ryzencap.nixosModules.ryzencap
];

#Example config
services.ryzencap = {
  enable = true;
  tctl_limit = 85;
  quiet = {
    enable = true;
    unplugged = {
      enable = true;
      stapm_limit = 7000;
      fast_limit = 7000;
      slow_limit = 7000;
      apu_slow_limit = 20000;
    };
  };
};
```

For information about the options check the 
[ryzenadj repo](https://github.com/FlyGoat/RyzenAdj).
