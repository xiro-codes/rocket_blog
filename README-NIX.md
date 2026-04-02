# NixOS Deployment Guide

This project has been ported from Docker to native Nix packages and NixOS modules.

## Features
- **Nix Flake**: Native compilation using `rustPlatform.buildRustPackage`.
- **NixOS Module**: Exposes `services.rocket-blog` for simple, declarative deployment on NixOS.
- **Nginx Proxy**: Automatically manages Nginx reverse proxy configurations for both `blog` and `worktime` endpoints.
- **Systemd Integration**: Runs securely using `DynamicUser` and automatically restarts on failure.
- **PostgreSQL Management**: Can optionally configure and manage the required PostgreSQL database and users.

## Usage

### 1. Using the NixOS Module

To use this on an existing NixOS machine, add the flake to your inputs and import the module:

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rocket-blog.url = "github:your-repo/rocket-blog";
  };

  outputs = { self, nixpkgs, rocket-blog, ... }: {
    nixosConfigurations.my-server = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        rocket-blog.nixosModules.default
        ({ config, pkgs, ... }: {
          services.rocket-blog = {
            enable = true;
            domain = "myblog.example.com";
            
            # The worktime endpoint automatically defaults to worktime.myblog.example.com
            # worktimeDomain = "tracker.example.com";
            
            # Optionally let the module configure the postgres database for you:
            manageDatabase = true;
            
            # Reference a local file containing ROCKET_SECRET_KEY=...
            # secretKeyFile = "/path/to/secret.env";
          };
        })
      ];
    };
  };
}
```

### 2. Using as a Declarative NixOS Container

If you'd like to run it inside an isolated NixOS container (instead of directly on the host system), you can define it using NixOS's declarative `containers` option:

```nix
# In your host's configuration.nix
containers.rocket-blog = {
  autoStart = true;
  privateNetwork = true;
  hostAddress = "192.168.100.10";
  localAddress = "192.168.100.11";
  
  config = { config, pkgs, ... }: {
    imports = [ inputs.rocket-blog.nixosModules.default ];
    
    services.rocket-blog = {
      enable = true;
      domain = "blog.localhost";
      manageDatabase = true;
    };
    
    # Configure networking, firewalls, etc. inside the container here
    networking.firewall.allowedTCPPorts = [ 80 ];
  };
};
```

### 3. Local Nix Run

You can build and run the services directly via Nix without installing them to your system profile:

```bash
# Build the package
nix build

# Start the development shell
nix develop
```

## How the Module Works

The `nixosModules.default` (found in `nix/module.nix`) does the following:
1. Copies the compiled Rust application (including `templates` and `static` files) to the module's package path.
2. Generates two Systemd services: `rocket-blog` and `rocket-worktime`.
3. Optionally provisions PostgreSQL and configures the `rocket_blog` database user.
4. Generates an Nginx config pointing to the respective `blogPort` (default 8000) and `worktimePort` (default 8001).
