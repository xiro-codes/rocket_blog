{ self }:
{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.rocket-forge;
  pkg = self.packages.${pkgs.system}.default;
in
{
  options.services.rocket-forge = {
    package = mkOption {
      type = types.package;
      default = pkg;
      description = "The package to use for the rocket-forge services.";
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

    blog = {
      enable = mkEnableOption "Rocket Blog Service";
      port = mkOption { type = types.port; default = 8000; };
      domain = mkOption { type = types.str; default = "_"; };
    };

    worktime = {
      enable = mkEnableOption "Rocket Worktime Service";
      port = mkOption { type = types.port; default = 8001; };
      domain = mkOption { type = types.str; default = "_"; };
    };

    portfolio = {
      enable = mkEnableOption "Rocket Portfolio Service";
      port = mkOption { type = types.port; default = 8002; };
      domain = mkOption { type = types.str; default = "_"; };
    };

    handyman = {
      enable = mkEnableOption "Rocket Handyman Service";
      port = mkOption { type = types.port; default = 8003; };
      domain = mkOption { type = types.str; default = "_"; };
    };
  };

  config = mkIf (cfg.blog.enable || cfg.worktime.enable || cfg.portfolio.enable || cfg.handyman.enable) {
    services.postgresql = mkIf cfg.manageDatabase {
      enable = true;
      dataDir = mkIf (cfg.databaseDataDir != null) cfg.databaseDataDir;
      ensureDatabases = [ "rocket_blog" ];
      ensureUsers = [{
        name = "rocket_blog";
        ensureDBOwnership = true;
      }];
      authentication = pkgs.lib.mkOverride 10 ''
        local   all             all                                     trust
        host    all             all             127.0.0.1/32            trust
        host    all             all             ::1/128                 trust
      '';
    };

    services.redis.servers."".enable = true;

    systemd.services.rocket-blog = mkIf cfg.blog.enable {
      description = "Rocket Blog Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ] ++ (if cfg.manageDatabase then [ "postgresql.service" ] else [ ]);
      environment = {
        ROCKET_PROFILE = cfg.rocketProfile;
        ROCKET_PORT = toString cfg.blog.port;
        ROCKET_ADDRESS = "127.0.0.1";
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

    systemd.services.rocket-worktime = mkIf cfg.worktime.enable {
      description = "Rocket Worktime Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ] ++ (if cfg.manageDatabase then [ "postgresql.service" ] else [ ]);
      environment = {
        ROCKET_PROFILE = cfg.rocketProfile;
        ROCKET_PORT = toString cfg.worktime.port;
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

    systemd.services.rocket-portfolio = mkIf cfg.portfolio.enable {
      description = "Rocket Portfolio Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      environment = {
        ROCKET_PROFILE = cfg.rocketProfile;
        ROCKET_PORT = toString cfg.portfolio.port;
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

    systemd.services.rocket-handyman = mkIf cfg.handyman.enable {
      description = "Rocket Handyman Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      environment = {
        ROCKET_PROFILE = cfg.rocketProfile;
        ROCKET_PORT = toString cfg.handyman.port;
        ROCKET_ADDRESS = "127.0.0.1";
      };
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/handyman";
        WorkingDirectory = cfg.workingDirectory;
        EnvironmentFile = mkIf (cfg.secretKeyFile != null) cfg.secretKeyFile;
        Restart = "always";
        DynamicUser = true;
      };
    };

    services.nginx = {
      enable = true;
      virtualHosts = 
        let
          mkVhost = port: {
            locations."/" = {
              proxyPass = "http://127.0.0.1:${toString port}";
              extraConfig = ''
                proxy_set_header Host $host;
                proxy_set_header X-Real-IP $remote_addr;
                proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
                proxy_set_header X-Forwarded-Proto $scheme;
              '';
            };
          };
        in
          (optionalAttrs cfg.blog.enable { "${cfg.blog.domain}" = mkVhost cfg.blog.port; })
          // (optionalAttrs cfg.worktime.enable { "${cfg.worktime.domain}" = mkVhost cfg.worktime.port; })
          // (optionalAttrs cfg.portfolio.enable { "${cfg.portfolio.domain}" = mkVhost cfg.portfolio.port; })
          // (optionalAttrs cfg.handyman.enable { "${cfg.handyman.domain}" = mkVhost cfg.handyman.port; });
    };
  };
}
