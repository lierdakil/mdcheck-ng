self:
{
  name = "module config";
  nodes.machine = {
    imports = [ self.nixosModules.default ];
    services.mdcheck-ng = {
      enable = true;
      logLevel = "trace";
      global.start = "Sun#1";
      global.max_run_duration = "30s";
      devices = {
        md127.nice = 15;
        md127.ionice = "idle";
        md126.nice = 14;
        md126.ionice.best_effort = 7;
        md126.max_run_duration = "12h";
      };
    };
    system.stateVersion = "24.11";
  };
  testScript = ''
    machine.succeed("systemctl start mdcheck-ng.service")
    out = machine.succeed("systemctl cat mdcheck-ng.service")
    exec_line = next((line for line in out.splitlines() if line.startswith("ExecStart=")))
    file = exec_line.split()[1]
    out = machine.succeed(f"cat {file}").strip()
    expect = """
    max_run_duration = "30s"
    start = "Sun#1"

    [md126]
    max_run_duration = "12h"
    nice = 14
    [md126.ionice]
    best_effort = 7

    [md127]
    ionice = "idle"
    nice = 15
    """.strip()
    print(machine.succeed(f"diff -u <(echo '{expect}') <(echo '{out}')"))
    assert out == expect
  '';
}
