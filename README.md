# gits

A git wrapper with interactive commit and branch selectors.
Run common git operations quickly, using only the keyboard.

## Installation

```sh
cargo install --path .
```

If `~/.cargo/bin` is on your `PATH`, you can run it as `gits <subcommand>`.

### Checking your PATH

```sh
# Check that gits is on your PATH
gits --help
```

If it is not, add `~/.cargo/bin` to your `PATH`.

```sh
# Add to .bashrc / .zshrc
export PATH="$HOME/.cargo/bin:$PATH"
```

## Usage

```sh
gits <subcommand> [options]
```

In the selector, move with `j`/`k` or the arrow keys, confirm with `Enter` or `l`, and cancel with `Esc` / `h` / `q` / `Ctrl-C`.

---

## Subcommands

### `show` — Show a commit

Interactively select a commit from the list and run `git show` on it.

```sh
gits show
gits show --stat
gits show -p --name-only
```

### `diff` — Show changes between commits

Select a base commit and then a target commit, and run `git diff` between them.

```sh
gits diff
gits diff --stat
gits diff --base HEAD         # Fix the base and select only the target
```

### `switch` — Switch branches

Select a branch from the list and run `git switch`.

```sh
gits switch
gits switch --detach
```

### `merge` — Merge a branch

Select a branch from the list and run `git merge`.

```sh
gits merge
gits merge --no-ff
```

### `rebase` — Rebase onto a branch

Select a branch from the list and run `git rebase`.

```sh
gits rebase
gits rebase --interactive     # Run git rebase -i with the selected branch
```

### `commit` — Print a commit hash

Select a commit from the list and print its hash to standard output.
Use it in combination with other commands.

```sh
git rebase -i $(gits commit)
git diff $(gits commit) HEAD
git cherry-pick $(gits commit)
```

### `branch` — Print a branch name

Select a branch from the list and print its name to standard output.

```sh
git switch $(gits branch)
git diff $(gits branch)
git merge --no-ff $(gits branch)
```

---

## Key bindings

| Key                          | Action                 |
| ---------------------------- | ---------------------- |
| `j` / `↓`                    | Next item              |
| `k` / `↑`                    | Previous item          |
| `Enter` / `l`                | Confirm selection      |
| `Esc` / `h` / `q` / `Ctrl-C` | Cancel (exit code 130) |

### Custom key bindings

Key bindings can be changed in `gits/config.toml` under your config directory:

| OS      | Path                                                  |
| ------- | ----------------------------------------------------- |
| Linux   | `~/.config/gits/config.toml`                          |
| macOS   | `~/Library/Application Support/gits/config.toml`      |
| Windows | `%APPDATA%\gits\config.toml`                          |

```toml
[keymap]
up = ["k", "up", "ctrl-p"]
down = ["j", "down", "ctrl-n"]
confirm = ["enter", "l"]
cancel = ["esc", "q", "ctrl-c"]
```

- All four of `up`, `down`, `confirm` and `cancel` must be present. If any is missing or the file cannot be parsed, the defaults are used for everything.
- A non-empty list replaces that action's defaults. An empty list (`[]`) keeps the defaults.
- Keys are either a single character or one of `up`, `down`, `left`, `right`, `enter`, `esc`, `space`, `tab`, `backspace`, `delete`, `home`, `end`, `pageup`, `pagedown`.
- Prefix keys with `ctrl-`, `alt-`, `shift-` or `cmd-` (or `+` instead of `-`, e.g. `ctrl+n`) to add modifiers.
- Unrecognized keys are ignored.
