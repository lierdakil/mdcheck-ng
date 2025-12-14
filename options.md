## services\.mdcheck-ng\.enable



Whether to enable mdcheck-ng service\.



*Type:*
boolean



*Default:*
` false `



*Example:*
` true `



## services\.mdcheck-ng\.devices

Per-device overrides



*Type:*
attribute set of (submodule)



*Default:*
` { } `



*Example:*

```
{
  md127 = {
    continue = "Sat";
    max_run_duration = "6h";
    start = "Sat#1";
  };
}
```



## services\.mdcheck-ng\.devices\.\<name>\.continue



Cron string to define when a scrub can continue, in the same format as ` start `\.



*Type:*
null or string



*Default:*
` null `



*Example:*
` "Sun" `



## services\.mdcheck-ng\.devices\.\<name>\.ionice



Either ` "idle" `, or ` { best_effort = lvl; } `, or ` { realtime = lvl; } `, where lvl is between 0 and 7



*Type:*
null or value “idle” (singular enum) or (submodule) or (submodule)



*Default:*
` null `



*Example:*
` "-c2 -n7" `



## services\.mdcheck-ng\.devices\.\<name>\.max_run_duration



Maximum duration for a single run\. Used to limit scrub time\.
Unspecified means unlimited\. Accepts ` humantime ` strings, see
[https://docs\.rs/humantime/latest/humantime/fn\.parse_duration\.html](https://docs\.rs/humantime/latest/humantime/fn\.parse_duration\.html)
for the exact syntax\.



*Type:*
null or string



*Default:*
` null `



*Example:*
` "6h 30m" `



## services\.mdcheck-ng\.devices\.\<name>\.nice



Nice level for the scrub process



*Type:*
null or integer between -20 and 19 (both inclusive)



*Default:*
` null `



*Example:*
` 15 `



## services\.mdcheck-ng\.devices\.\<name>\.start



Cron string to define when a scrub can start, in the croner format\.
See [https://docs\.rs/croner/latest/croner/\#pattern](https://docs\.rs/croner/latest/croner/\#pattern) for exact syntax\. Any fields not
specified will be assumed to be ` * `, so you could specify just ` Sun#1 ` to run on the
first Sunday of the month\.



*Type:*
null or string



*Default:*
` null `



*Example:*
` "Sun#1" `



## services\.mdcheck-ng\.global



Global options



*Type:*
submodule



*Default:*
` { } `



## services\.mdcheck-ng\.global\.continue



Cron string to define when a scrub can continue, in the same format as ` start `\.



*Type:*
null or string



*Default:*
` null `



*Example:*
` "Sun" `



## services\.mdcheck-ng\.global\.ionice



Either ` "idle" `, or ` { best_effort = lvl; } `, or ` { realtime = lvl; } `, where lvl is between 0 and 7



*Type:*
null or value “idle” (singular enum) or (submodule) or (submodule)



*Default:*
` null `



*Example:*
` "-c2 -n7" `



## services\.mdcheck-ng\.global\.max_run_duration



Maximum duration for a single run\. Used to limit scrub time\.
Unspecified means unlimited\. Accepts ` humantime ` strings, see
[https://docs\.rs/humantime/latest/humantime/fn\.parse_duration\.html](https://docs\.rs/humantime/latest/humantime/fn\.parse_duration\.html)
for the exact syntax\.



*Type:*
null or string



*Default:*
` null `



*Example:*
` "6h 30m" `



## services\.mdcheck-ng\.global\.nice



Nice level for the scrub process



*Type:*
null or integer between -20 and 19 (both inclusive)



*Default:*
` null `



*Example:*
` 15 `



## services\.mdcheck-ng\.global\.start



Cron string to define when a scrub can start, in the croner format\.
See [https://docs\.rs/croner/latest/croner/\#pattern](https://docs\.rs/croner/latest/croner/\#pattern) for exact syntax\. Any fields not
specified will be assumed to be ` * `, so you could specify just ` Sun#1 ` to run on the
first Sunday of the month\.



*Type:*
null or string



*Default:*
` null `



*Example:*
` "Sun#1" `



## services\.mdcheck-ng\.logLevel



Log level



*Type:*
one of “trace”, “debug”, “info”, “warn”, “error”



*Default:*
` "error" `



*Example:*
` "info" `



## services\.mdcheck-ng\.runSchedule



When to run the service\. Must fall within start and continue\. Systemd OnCalendar format\.



*Type:*
string



*Default:*
` "daily" `



*Example:*
` "Sun *-*-* 01:00" `


