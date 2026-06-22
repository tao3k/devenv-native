{
  __inputs__,
  ...
}:
let
  processScript = processName: scriptName: ''
    ROOT_DIR="''${PRJ_ROOT:-''${DEVENV_ROOT:-$(pwd)}}"
    exec bash "$ROOT_DIR/scripts/runtime/processes/${processName}/${scriptName}.sh"
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

    qianji-server = {
      exec = processEntrypoint "qianji-server";
      process-compose = {
        depends_on = {
          valkey.condition = "process_healthy";
        };
        readiness_probe = {
          exec.command = processHealthcheck "qianji-server";
          initial_delay_seconds = 5;
          period_seconds = 3;
          timeout_seconds = 3;
          failure_threshold = 30;
        };
      };
    };

    carfox.exec = processEntrypoint "carfox";

    vllm-sr = {
      exec = processEntrypoint "vllm-sr";
      process-compose = {
        availability = {
          restart = "no";
        };
        readiness_probe = {
          exec.command = processHealthcheck "vllm-sr";
          initial_delay_seconds = 5;
          period_seconds = 3;
          timeout_seconds = 3;
          failure_threshold = 40;
        };
      };
    };

    # Wendao Phase 7.6 Integrated Services
    wendao-analyzer = {
      exec = processEntrypoint "wendao-analyzer";
      process-compose = {
        depends_on = {
          wendao-gateway.condition = "process_healthy";
        };
        readiness_probe = {
          exec.command = processHealthcheck "wendao-analyzer";
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

    "wendao-ai" = {
      exec = processEntrypoint "wendao-ai";
      process-compose = {
        depends_on = {
          qianji-server.condition = "process_healthy";
          wendao-gateway.condition = "process_healthy";
        };
        readiness_probe = {
          exec.command = processHealthcheck "wendao-ai";
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
          vllm-sr.condition = "process_healthy";
        };
        readiness_probe = {
          exec.command = processHealthcheck "wendao-gateway";
          initial_delay_seconds = 30;
          period_seconds = 5;
          timeout_seconds = 2;
          failure_threshold = 120;
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

    wendao-semantic-refresh = {
      exec = processEntrypoint "wendao-semantic-refresh";
      process-compose = {
        readiness_probe = {
          exec.command = processHealthcheck "wendao-semantic-refresh";
          initial_delay_seconds = 5;
          period_seconds = 10;
          timeout_seconds = 3;
          failure_threshold = 6;
        };
      };
    };
  };
}
