# FL Plugin Sorter

A command-line tool to sort FL Studio plugin files (`.fst` files) into folders in its plugin database.

This is done by creating plugin group definitions (`.toml` files) that specify the name
of a group of plugins, along with a list of plugins that should be included in that group.

## Usage

In order to begin sorting plugins, you need to have at least one plugin group defined.

You can create a plugin group in the `effect` or `generator` folders in `~/.config/flsorter/`.
They can be named anything, but the file extension must be `toml`.

The format for a plugin group file is:

```toml
group = "Plugin group name"
plugins = [
    "Plugin name",
    "Another plugin name"
]
```
