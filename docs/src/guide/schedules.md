# Schedules

Schedules let you run tasks on a timer — either at specific times (cron) or at regular intervals.

## Interval Schedules

```
schedule <name> {
    every <duration>

    # task body
}
```

Duration format: `<number><unit>` where unit is `s` (seconds), `m` (minutes), `h` (hours), `d` (days).

```
schedule reminder {
    every 1h

    let channel = discord.get_channel("general")
    send "Time to take a break!" in channel channel
}
```

## Cron Schedules

```
schedule <name> {
    cron "<minute> <hour> <day> <month> <weekday>"

    # task body
}
```

Standard 5-field cron syntax:

```
schedule daily_backup {
    cron "0 3 * * *"      # Every day at 3:00 AM
    std.log("Running daily backup")
}

schedule weekly_report {
    cron "30 9 * * 1"     # Every Monday at 9:30 AM
    # generate report
}
```

| Field | Values | Description |
|-------|--------|-------------|
| Minute | 0-59 | Minute of hour |
| Hour | 0-23 | Hour of day |
| Day | 1-31 | Day of month |
| Month | 1-12 | Month of year |
| Weekday | 0-7 (0/7 = Sun) | Day of week |

## Using Events with Schedules

Schedules can trigger events:

```
schedule hourly {
    cron "0 * * * *"
    event "hourly_tick"

    let guild = discord.get_guild("main")
    let channel = discord.get_channel("updates")
    send "Hourly update tick" in channel channel
}
```

## Multiple Schedules

```
schedule stats {
    every 30m
    # Collect and log stats
}

schedule cleanup {
    cron "0 0 * * 0"      # Weekly on Sunday
    # Clean up old data
}

schedule backup {
    cron "0 2 * * *"      # Daily at 2 AM
    db.backup()
}
```
