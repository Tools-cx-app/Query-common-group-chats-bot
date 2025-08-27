# Query Common Group Chats Bot  
A combination of a Telegram bot and UserBot to count and query the common members of multiple groups.

You can add or remove target groups in the configuration file at any time, or specify additional administrators.

---

## Highlights

- Supports querying common members of unlimited groups
- Interacts via bot commands, ensuring safety and ease of use
- Uses UserBot in the background to fetch complete member lists for more accurate data
- Supports multiple administrators with hot-reloading of configurations during runtime
- Implemented in pure Rust, with a small size, fast startup, and no external database dependencies

---

# Prerequisites

1. A Telegram Bot (created via @BotFather)
2. A regular Telegram account (acting as UserBot to fetch member lists)
3. Rust 1.73+ and Cargo installed

---

## Quick Start

### 1. Clone and Enter the Project

```bash
git clone https://github.com/Tools-cx-app/Query-common-group-chats-bot.git
cd query-common-group-chats-bot
```

### 2. First Run (Automatically Generates Template Configuration)

```bash
cargo run --release
```

The first run will generate two files in the `./` directory:
- `config.toml` —— Target groups, administrators, bot token, etc.
- `userbot.session` —— Session file for UserBot login persistence
- `bot.session` —— Session file for Bot login persistence

> Note: During the first run, the terminal will prompt you to receive a verification code via UserBot. Follow the instructions to complete the process.

### 3. Modify the Configuration

Open `config.toml` and fill in or modify as needed:

```toml
groups = []
admins = []
```

No need to restart; the configuration will automatically hot-reload during the next command.

---

## Usage

Command	Description	
`/addadmin <uid>`	Add an administrator (only available to super administrators)	
`/addgroup <uid>`	Add a group chat (only available to super administrators)	

---

## Frequently Asked Questions (FAQ)

### 1. How to Get Group Chat ID?

Group Chat ID

### 2. What to Do If UserBot Is Banned?

Delete the `userbot.session` file and log in again; if banned for frequent member fetching, reduce the query frequency or run only during working hours.

### 3. Configuration File Format Error Prevents Startup?

Don't worry, a new template will be automatically generated.

### 4. How to Persist Logs?

Set the environment variable `RUST_LOG=info` and redirect the output:

```bash
RUST_LOG=info cargo run --release > bot.log 2>&1
```

---

# Tips
For the first run, you need to modify the API_ID, API_HASH and SUPER_ADMIN in the src/defs.rs file.
