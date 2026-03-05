# ferrotype

A terminal typing practice app. Load any text file and type through it line by line while a virtual keyboard tracks your keystrokes.

## Build

```
cargo build --release
```

## Run

```
cargo run
```

## Controls

| Key | Action |
|---|---|
| `Ctrl-F` | Open file picker |
| `Enter` | Confirm file path |
| `Esc` | Cancel search / quit |

## Usage

1. Launch the app
2. Press `Ctrl-F` and type the path to a text file (absolute or relative to the working directory)
3. Press `Enter` to load it
4. Type the text shown — correct keystrokes advance the cursor, incorrect ones are highlighted
5. When the file is done, your accuracy is displayed
6. Press `Ctrl-F` again to load another file

A sample file is included:

```
cargo run
# then Ctrl-F → sample.txt → Enter
```
