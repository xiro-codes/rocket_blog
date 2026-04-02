{ self }:
{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.rocket-blog;
  pkg = self.packages.${pkgs.system}.rocket-blog;
in {
  options.services.rocket-blog = {
    enable = mkEnableOption "Rocket Blog & Worktime Service";

    package = mkOption {
      type = types.package;
      default = pkg;
      description = "The package to use for the rocket-blog service.";
    };

    domain = mkOption {
      type = types.str;
      default = "localhost";
      description = "The primary domain for the blog (e.g., blog.example.com).";
    };

    worktimeDomain = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "The domain for the worktime app. Defaults to worktime.<domain>.";
    };

    blogPort = mkOption {
      type = types.port;
      default = 8000;
      description = "Port for the blog service to listen on.";
    };

    worktimePort = mkOption {
      type = types.port;
      default = 8001;
      description = "Port for the worktime service to listen on.";
    };

    databaseUrl = mkOption {
      type = types.str;
      default = "postgres://rocket_blog:rocket_blog@localhost/rocket_blog";
      description = "Database connection string.";
    };

    manageDatabase = mkOption {
      type = types.bool;
      default = false;
      description = "Whether to automatically configure a local PostgreSQL database for this service.";
    };
    
    secretKeyFile = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = "Path to a file containing ROCKET_SECRET_KEY=... for session encryption.";
    };
  };

  config = mkIf cfg.enable {
    # Determine the worktime domain
    _module.args.actualWorktimeDomain = if cfg.worktimeDomain != null then cfg.worktimeDomain else "worktime.${cfg.domain}";

    # Configure PostgreSQL if requested
    services.postgresql = mkIf cfg.manageDatabase {
      enable = true;
      ensureDatabases = [ "rocket_blog" ];
      ensureUsers = [{
        name = "rocket_blog";
        ensureDBOwnership = true;
        # For simplicity in local container dev, we can set a password via init script, or rely on ident auth.
        # Here we just use password auth since the URL has a password:
      }];
      authentication = pkgs.lib.mkOverride 10 ''
        # "local" is for Unix domain socket connections only
        local   all             all                                     trust
        # IPv4 local connections:
        host    all             all             127.0.0.1/32            trust
        # IPv6 local connections:
        host    all             all             ::1/128                 trust
      '';
    };

    # Systemd Service for Blog
    systemd.services.rocket-blog = {
      description = "Rocket Blog Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ] ++ (if cfg.manageDatabase then [ "postgresql.service" ] else []);
      
      environment = {
        ROCKET_PROFILE = "release";
        ROCKET_PORT = toString cfg.blogPort;
        ROCKET_ADDRESS = "127.0.0.1";
        ROCKET_DATABASES__SEA_ORM__URL = cfg.databaseUrl;
      };

      serviceConfig = {
        ExecStart = "${cfg.package}/bin/blog";
        WorkingDirectory = "${cfg.package}/share/rocket-blog";
        EnvironmentFile = mkIf (cfg.secretKeyFile != null) cfg.secretKeyFile;
        Restart = "always";
        DynamicUser = true;
      };
    };

    # Systemd Service for Worktime Tracker
    systemd.services.rocket-worktime = {
      description = "Rocket Worktime Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ] ++ (if cfg.manageDatabase then [ "postgresql.service" ] else []);
      
      environment = {
        ROCKET_PROFILE = "release";
        ROCKET_PORT = toString cfg.worktimePort;
        ROCKET_ADDRESS = "127.0.0.1";
        ROCKET_DATABASES__SEA_ORM__URL = cfg.databaseUrl;
      };

      serviceConfig = {
        ExecStart = "${cfg.package}/bin/worktime";
        WorkingDirectory = "${cfg.package}/share/rocket-blog";
        EnvironmentFile = mkIf (cfg.secretKeyFile != null) cfg.secretKeyFile;
        Restart = "always";
        DynamicUser = true;
      };
    };

    # Nginx Configuration
    services.nginx = {
      enable = true;
      
      virtualHosts.${cfg.domain} = {
        locations."/" = {
          proxyPass = "http://127.0.0.1:${toString cfg.blogPort}";
          extraConfig = ''
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
          '';
        };
      };
      
      virtualHosts.${if cfg.worktimeDomain != null then cfg.worktimeDomain else "worktime.${cfg.domain}"} = {
        locations."/" = {
          proxyPass = "http://127.0.0.1:${toString cfg.worktimePort}";
          extraConfig = ''
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
          '';
        };
      };
    };
  };
}
