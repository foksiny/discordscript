# Embeds & Components

## Embeds

Rich embeds are defined inline:

```
cmd info {
    slash true
    embed {
        title "Bot Information"
        description "Everything you need to know"
        color "blue"
        field "Version" value "1.0.0" inline true
        field "Library" value "DiscordScript" inline true
        footer "Powered by DiscordScript"
        timestamp true
    }
}
```

### Embed Properties

| Property | Description |
|----------|-------------|
| `title` | Embed title (string) |
| `description` | Embed description (string) |
| `color` | Color name or hex: `"blue"`, `"green"`, `"red"`, `"#ff0000"` |
| `field` | Named field with `name`, `value`, `inline` |
| `footer` | Footer text |
| `image` | Image URL |
| `thumbnail` | Thumbnail URL |
| `author` | Author name with optional icon |
| `timestamp` | Show current time (boolean) |

### Dynamic Embeds

Embed values can be variables:

```
let title = "Stats for {user.name}"
let color = if user.is_premium { "gold" } else { "grey" }

embed {
    title title
    color color
    field "Messages" value "{msg_count}" inline true
    field "Voice Time" value "{voice_minutes}m" inline true
}
```

## Rows & Buttons

```
row {
    button "Primary" {
        style primary
        label "Click Me"
        custom_id "btn_click"
    }
    button "Danger" {
        style danger
        label "Delete"
        custom_id "btn_delete"
    }
}
```

### Button Styles

| Style | Usage |
|-------|-------|
| `primary` | Blue, main action |
| `secondary` | Grey, secondary action |
| `success` | Green, confirm action |
| `danger` | Red, destructive action |
| `link` | URL link (requires `url`) |

### Button Properties

| Property | Description |
|----------|-------------|
| `label` | Button text |
| `style` | Button style (see above) |
| `custom_id` | Unique identifier for interaction |
| `url` | URL for link buttons |
| `disabled` | Disable the button (boolean) |
| `emoji` | Emoji to display |

## Select Menus

```
select_menu {
    custom_id "color_picker"
    placeholder "Choose a color"
    min_values 1
    max_values 1
    option "Red" {
        value "red"
        description "The color red"
        emoji "🟥"
    }
    option "Green" {
        value "green"
        description "The color green"
        emoji "🟩"
    }
    option "Blue" {
        value "blue"
        description "The color blue"
        emoji "🟦"
    }
}
```

## Modals

```
modal {
    title "Feedback Form"
    custom_id "feedback"
    text_input "Name" {
        style short
        placeholder "Your name"
        required true
    }
    text_input "Message" {
        style paragraph
        placeholder "Your feedback"
        required true
        max_length 1000
    }
}
```

### Text Input Styles

| Style | Description |
|-------|-------------|
| `short` | Single line |
| `paragraph` | Multi-line |

## Pagination

```
cmd list {
    slash true
    let items = db.find_all("users")

    paginate items per_page 10 {
        embed {
            title "Users"
            description "Page {page} of {total_pages}"
            field "Name" value "{item.name}" inline true
            field "Score" value "{item.score}" inline true
        }
    }
}
```

The paginate block takes an embed template and handles navigation buttons automatically.
