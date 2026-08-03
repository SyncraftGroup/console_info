# console_info

Shared console UI toolkit for the Syncraft ecosystem.

## Features

- **Structured terminal output** with colored labels: `[STEP]`, `[OK]`, `[INFO]`, `[WARN]`, `[FAIL]`
- **Unicode box-drawing** for multi-line messages (info, warning, error, plain)
- **Interactive prompts**: text input, password (masked), yes/no confirmation, single-select lists
- **Locale detection** (Chinese `Zh` / English `En`) with a `msgs!` macro for bilingual strings
- **Themed `inquire` prompts** with Syncraft's color scheme

## Installation

```toml
[dependencies]
console_info = { git = "https://github.com/syncraft/console_info" }
```

## Usage

### Status messages

```rust
use console_info::Ui;

let ui = Ui::get();

ui.step("Initializing...");
ui.ok("Configuration loaded");
ui.info("Server listening on 0.0.0.0:8080");

// Multi-line messages are rendered inside a bordered box
ui.warn("Could not reach update server\nCheck your network connection");
ui.error("Fatal: Missing required dependency\nRun `setup` first");
```

### Interactive prompts

```rust
use console_info::{Ui, prompt_input, prompt_confirm, prompt_select};

let ui = Ui::get();
let name = ui.input("Enter your name:")?;
let proceed = ui.confirm("Continue?")?;
let choice = ui.select("Pick an option:", vec!["A", "B", "C"])?;
```

### Bilingual strings

```rust
use console_info::{msgs, Ui};

let greeting = msgs! { En: "Hello!", Zh: "你好！" };
let ui = Ui::get();
ui.info(&greeting);
```

### Theming inquire

```rust
use console_info::configure_inquire;

configure_inquire();
// Now all inquire prompts (Text, Select, Confirm, etc.) use Syncraft styling
```

## Requirements

- Rust **1.85.0** or later (edition 2024)

## License

MIT
