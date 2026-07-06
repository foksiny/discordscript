# Modules & Imports

Modules let you split your bot into multiple files for better organization.

## Importing Modules

```
import "modules/utility.ds"
import "modules/admin.ds" as admin
import "modules/help.ds" use ping, help
```

### Import Variants

| Syntax | Effect |
|--------|--------|
| `import "file.ds"` | Runs the file, all top-level commands/events are loaded |
| `import "file.ds" as ns` | Namespaced — items are accessed via `ns.name` |
| `import "file.ds" use a, b` | Selective import — only specific items |

## Exporting

Commands and events in a module file are automatically available to the importer by default. Use `export` for explicit exports:

```
# modules/admin.ds
export cmd ban {
    slash true
    permission "ban_members"
    param user { type user }
    reply "Banned {user.tag}"
}

export cmd kick {
    slash true
    permission "kick_members"
    param user { type user }
    reply "Kicked {user.tag}"
}
```

## Module Discovery

DiscordScript automatically discovers modules in the `modules/` directory, events in `events/`, and schedules in `schedules/`.

## File Structure

```
my-bot/
├── main.ds                # Entry point
├── config.ds              # Bot configuration
├── modules/
│   ├── utility.ds         # Utility commands
│   ├── admin.ds           # Admin commands
│   └── fun.ds             # Fun commands
├── events/
│   ├── welcome.ds         # Welcome events
│   └── logging.ds         # Logging events
└── schedules/
    ├── backup.ds          # Backup schedules
    └── stats.ds           # Stats schedules
```
