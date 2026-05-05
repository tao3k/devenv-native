{
  __inputs__,
  ...
}:
let
  processScript =
    processName: scriptName:
    ''
      ROOT_DIR="''${PRJ_ROOT:-''${DEVENV_ROOT:-$(pwd)}}"
      exec bash "$ROOT_DIR/scripts/channel/processes/${processName}/${scriptName}.sh"
    '';
  processEntrypoint = processName: processScript processName "entrypoint";
  processHealthcheck = processName: processScript processName "healthcheck";
in
{
  packages = [
    __inputs__.packages.capfox
  ];
  process.manager.implementation = "process-compose";
  processes = {
    valkey = {
      exec = processEntrypoint "valkey";
      process-compose = {
        readiness_probe = {
          exec.command = processHealthcheck "valkey";
          initial_delay_seconds = 5;
          period_seconds = 3;
          timeout_seconds = 4;
          failure_threshold = 30;
        };
      };
    };

    carfox.exec = processEntrypoint "carfox";
    agent.exec = processEntrypoint "agent";

    # Wendao Phase 7.6 Integrated Services
    wendao-document-extract = {
      exec = processEntrypoint "wendao-document-extract";
      process-compose = {
        depends_on = {
          wendao-gateway.condition = "process_healthy";
        };
        readiness_probe = {
          exec.command = processHealthcheck "wendao-document-extract";
          initial_delay_seconds = 5;
          period_seconds = 3;
          timeout_seconds = 4;
          failure_threshold = 40;
        };
      };
    };

    "wendao-frontend" = {
      exec = processEntrypoint "wendao-frontend";
      process-compose = {
        depends_on = {
          wendao-gateway.condition = "process_healthy";
        };
        readiness_probe = {
          exec.command = processHealthcheck "wendao-frontend";
          initial_delay_seconds = 5;
          period_seconds = 2;
          timeout_seconds = 3;
          failure_threshold = 30;
        };
      };
    };

    wendao-gateway = {
      exec = processEntrypoint "wendao-gateway";
      process-compose = {
        depends_on = {
          valkey.condition = "process_healthy";
        };
        readiness_probe = {
          exec.command = processHealthcheck "wendao-gateway";
          initial_delay_seconds = 15;
          period_seconds = 5;
          timeout_seconds = 2;
          failure_threshold = 30;
        };
      };
    };

    wendao-sentinel = {
      exec = processEntrypoint "wendao-sentinel";
      process-compose = {
        depends_on = {
          wendao-gateway.condition = "process_healthy";
        };
        readiness_probe = {
          exec.command = processHealthcheck "wendao-sentinel";
          initial_delay_seconds = 10;
          period_seconds = 5;
          timeout_seconds = 2;
          failure_threshold = 12;
        };
      };
    };

    wendaosearch-solver-demo = {
      exec = processEntrypoint "wendaosearch-solver-demo";
      process-compose = {
        readiness_probe = {
          exec.command = processHealthcheck "wendaosearch-solver-demo";
          initial_delay_seconds = 5;
          period_seconds = 2;
          timeout_seconds = 3;
          failure_threshold = 90;
        };
      };
    };

    wendaosearch-parser-summary = {
      exec = processEntrypoint "wendaosearch-parser-summary";
      process-compose = {
        readiness_probe = {
          exec.command = processHealthcheck "wendaosearch-parser-summary";
          initial_delay_seconds = 5;
          period_seconds = 2;
          timeout_seconds = 3;
          failure_threshold = 90;
        };
      };
    };
  };
}
