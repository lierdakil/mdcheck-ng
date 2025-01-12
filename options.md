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
    start = "* * 1-15 * * Sat";
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
` "* * 1-15 * * Sun" `



## services\.mdcheck-ng\.devices\.\<name>\.ionice



ionice CLI arguments specifying ionice class and level for the scrub process



*Type:*
null or string



*Default:*
` null `



*Example:*
` "-c2 -n7" `



## services\.mdcheck-ng\.devices\.\<name>\.nice



Nice level for the scrub process



*Type:*
null or 8 bit signed integer; between -128 and 127 (both inclusive)



*Default:*
` null `



*Example:*
` 15 `



## services\.mdcheck-ng\.devices\.\<name>\.start



Cron string to define when a scrub can start, in the croner format\.
See [https://docs\.rs/croner/latest/croner/\#pattern](https://docs\.rs/croner/latest/croner/\#pattern) for exact syntax, but note that
seconds are NOT optional\.



*Type:*
null or string



*Default:*
` null `



*Example:*
` "* * 1-15 * * Sun#1" `



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
` "* * 1-15 * * Sun" `



## services\.mdcheck-ng\.global\.ionice



ionice CLI arguments specifying ionice class and level for the scrub process



*Type:*
null or string



*Default:*
` null `



*Example:*
` "-c2 -n7" `



## services\.mdcheck-ng\.global\.nice



Nice level for the scrub process



*Type:*
null or 8 bit signed integer; between -128 and 127 (both inclusive)



*Default:*
` null `



*Example:*
` 15 `



## services\.mdcheck-ng\.global\.start



Cron string to define when a scrub can start, in the croner format\.
See [https://docs\.rs/croner/latest/croner/\#pattern](https://docs\.rs/croner/latest/croner/\#pattern) for exact syntax, but note that
seconds are NOT optional\.



*Type:*
null or string



*Default:*
` null `



*Example:*
` "* * 1-15 * * Sun#1" `



## services\.mdcheck-ng\.logLevel



Log level



*Type:*
one of “trace”, “debug”, “info”, “warn”, “error”



*Default:*
` "error" `



*Example:*
` "info" `



## services\.mdcheck-ng\.maxRunDuration



Maximum duration for a single run\. Can be used to limit scrub time,
instead of specifying ranges in ` start ` and ` continue `\. Accepts
` humantime ` strings, see [https://docs\.rs/humantime/latest/humantime/fn\.parse_duration\.html](https://docs\.rs/humantime/latest/humantime/fn\.parse_duration\.html)
for exact syntax\.



*Type:*
null or string



*Default:*
` null `



*Example:*
` "6h 30m" `



## services\.mdcheck-ng\.runSchedule



When to run the service\. Must fall within start and continue\. Systemd OnCalendar format\.



*Type:*
string



*Default:*
` "daily" `



*Example:*
` "Sun *-*-* 01:00" `


