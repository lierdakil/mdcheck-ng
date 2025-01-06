self:
{
  name = "module config";
  nodes.machine = {
    imports = [ self.nixosModules.default ];
    services.mdcheck-ng = {
      enable = true;
      maxRunDuration = "30s";
      logLevel = "trace";
      global.start = "* * * * * Sun#1";
      devices = {
        md127.nice = 15;
        md127.ionice = "-c3";
        md126.nice = 14;
        md126.ionice = "-c2 -n7";
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
    start = "* * * * * Sun#1"

    [md126]
    ionice = "-c2 -n7"
    nice = 14

    [md127]
    ionice = "-c3"
    nice = 15
    """.strip()
    print(machine.succeed(f"diff -u <(echo '{expect}') <(echo '{out}')"))
    assert out == expect
  '';
}
