{ self }:
{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.rocket-blog;
  pkg = self.packages.${pkgs.system}.rocket-blog;
in
{
  options.services.rocket-blog = {
    enable = mkEnableOption "Rocket Blog & Worktime Service";

    package = mkOption {
      type = types.package;
      default = pkg;
      description = "The package to use for the rocket-blog service.";
    };

    domain = mkOption {
      type = types.str;
      default = "_";
      description = "The primary domain for the applications (e.g., tdavis.dev).";
    };

    worktimeDomain = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "The domain for the worktime app. Defaults to worktime.<domain>.";
    };

    portfolioDomain = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "The domain for the portfolio app.";
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

    portfolioPort = mkOption {
      type = types.port;
      default = 8002;
      description = "Port for the portfolio service to listen on.";
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

    enableSeeding = mkOption {
      type = types.bool;
      default = false;
      description = "Whether to enable database seeding with sample data.";
    };

    databaseDataDir = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = "Path to store PostgreSQL database data. If null, uses the default.";
    };

    defaultAdminUsername = mkOption {
      type = types.str;
      default = "admin";
      description = "Default admin username to create if the database is empty.";
    };

    defaultAdminPassword = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Default admin password to create if the database is empty.";
    };

    secretKeyFile = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = "Path to a file containing ROCKET_SECRET_KEY=... for session encryption.";
    };

    rocketProfile = mkOption {
      type = types.str;
      default = "release";
      description = "The ROCKET_PROFILE to use (e.g. 'release' or 'debug').";
    };

    workingDirectory = mkOption {
      type = types.str;
      default = "${cfg.package}/share/rocket-blog";
      description = "Working directory for the services. Override this for development to point to local templates/static.";
    };
  };

  config = mkIf cfg.enable {
    # Determine the worktime domain
    _module.args.actualWorktimeDomain = if cfg.worktimeDomain != null then cfg.worktimeDomain else "worktime.${cfg.domain}";

    # Configure PostgreSQL if requested
    services.postgresql = mkIf cfg.manageDatabase {
      enable = true;
      dataDir = mkIf (cfg.databaseDataDir != null) cfg.databaseDataDir;
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

    # Configure Redis for session store
    services.redis.servers."".enable = true;

    # Systemd Service for Blog
    systemd.services.rocket-blog = {
      description = "Rocket Blog Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ] ++ (if cfg.manageDatabase then [ "postgresql.service" ] else [ ]);

      environment = {
        ROCKET_PROFILE = cfg.rocketProfile;
        ROCKET_PORT = toString cfg.blogPort;
        ROCKET_ADDRESS = "0.0.0.0";
        ROCKET_DATABASES__SEA_ORM__URL = cfg.databaseUrl;
        DEFAULT_ADMIN_USERNAME = cfg.defaultAdminUsername;
        ENABLE_SEEDING = if cfg.enableSeeding then "true" else "false";
      } // lib.optionalAttrs (cfg.defaultAdminPassword != null) {
        DEFAULT_ADMIN_PASSWORD = cfg.defaultAdminPassword;
      };

      serviceConfig = {
        ExecStart = "${cfg.package}/bin/blog";
        WorkingDirectory = cfg.workingDirectory;
        EnvironmentFile = mkIf (cfg.secretKeyFile != null) cfg.secretKeyFile;
        Restart = "always";
        DynamicUser = true;
      };
    };

    # Systemd Service for Worktime Tracker
    systemd.services.rocket-worktime = {
      description = "Rocket Worktime Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ] ++ (if cfg.manageDatabase then [ "postgresql.service" ] else [ ]);

      environment = {
        ROCKET_PROFILE = cfg.rocketProfile;
        ROCKET_PORT = toString cfg.worktimePort;
        ROCKET_ADDRESS = "127.0.0.1";
        ROCKET_DATABASES__SEA_ORM__URL = cfg.databaseUrl;
        DEFAULT_ADMIN_USERNAME = cfg.defaultAdminUsername;
        ENABLE_SEEDING = if cfg.enableSeeding then "true" else "false";
      } // lib.optionalAttrs (cfg.defaultAdminPassword != null) {
        DEFAULT_ADMIN_PASSWORD = cfg.defaultAdminPassword;
      };

      serviceConfig = {
        ExecStart = "${cfg.package}/bin/worktime";
        WorkingDirectory = cfg.workingDirectory;
        EnvironmentFile = mkIf (cfg.secretKeyFile != null) cfg.secretKeyFile;
        Restart = "always";
        DynamicUser = true;
      };
    };

    # Systemd Service for Portfolio
    systemd.services.rocket-portfolio = mkIf (cfg.portfolioDomain != null) {
      description = "Rocket Portfolio Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];

      environment = {
        ROCKET_PROFILE = cfg.rocketProfile;
        ROCKET_PORT = toString cfg.portfolioPort;
        ROCKET_ADDRESS = "127.0.0.1";
      };

      serviceConfig = {
        ExecStart = "${cfg.package}/bin/portfolio";
        WorkingDirectory = cfg.workingDirectory;
        EnvironmentFile = mkIf (cfg.secretKeyFile != null) cfg.secretKeyFile;
        Restart = "always";
        DynamicUser = true;
      };
    };

    # Nginx Configuration
    services.nginx = {
      enable = true;

      virtualHosts.${cfg.domain} = {
        locations."/" = mkIf (cfg.portfolioDomain != null) {
          proxyPass = "http://127.0.0.1:${toString cfg.portfolioPort}";
          extraConfig = ''
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
          '';
        };

        # Blog routes
        locations."/blog" = {
          proxyPass = "http://127.0.0.1:${toString cfg.blogPort}";
          extraConfig = ''
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
          '';
        };
        locations."/auth" = {
          proxyPass = "http://127.0.0.1:${toString cfg.blogPort}";
          extraConfig = ''
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
          '';
        };
        locations."/comment" = {
          proxyPass = "http://127.0.0.1:${toString cfg.blogPort}";
          extraConfig = ''
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
          '';
        };
        locations."/feed" = {
          proxyPass = "http://127.0.0.1:${toString cfg.blogPort}";
          extraConfig = ''
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
          '';
        };

        # Shared static folder (served by blog for simplicity)
        locations."/static" = {
          proxyPass = "http://127.0.0.1:${toString cfg.blogPort}";
          extraConfig = ''
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
          '';
        };

        # Worktime Tracker routes
        locations."/worklog" = mkIf (cfg.worktimeDomain != null) {
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
