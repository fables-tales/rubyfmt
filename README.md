# rubyfmt

A fast, opinionated Ruby formatter written in Rust.

- Try it online → [rubyfmt.run](https://rubyfmt.run)

Pronunciation: (en) "Ruby Format", (jp) ルビーフォーマット

## Quick Start

```bash
# Install
brew install rubyfmt

# Format a file in place
rubyfmt -i myfile.rb

# Format and print to stdout
rubyfmt myfile.rb
```

## Installation

### Homebrew

```bash
brew install rubyfmt
```

### Build from Source

1. Make sure you have Cargo installed
2. Run `cargo build --release`
3. Copy `target/release/rubyfmt-main` to somewhere on your path as `rubyfmt`

## Usage

### Command Line

| Command                  | Description                     |
| ------------------------ | ------------------------------- |
| `rubyfmt file.rb`        | Output formatted code to stdout |
| `rubyfmt -i file.rb`     | Format file in place            |
| `rubyfmt -c file.rb`     | Show diff of changes            |
| `cat file.rb \| rubyfmt` | Read from stdin                 |

You can also pass directories to format multiple files at once.

For the full command line interface, see `rubyfmt --help`.

### Header Comments

Control formatting on a per-file basis with header comments:

- `rubyfmt --header-opt-in` - Only format files with `# rubyfmt: true` at the top
- `rubyfmt --header-opt-out` - Skip files with `# rubyfmt: false` at the top

### Ignoring Files

Create a `.rubyfmtignore` file in your project root to exclude files from formatting. It uses the same syntax as `.gitignore`.

By default, rubyfmt also respects your `.gitignore`. Use `--include-gitignored` to format those files anyway.

## Editor Integration

### Visual Studio Code

The popular [`ruby-lsp` extension](https://marketplace.visualstudio.com/items?itemName=Shopify.ruby-lsp) has a [rubyfmt add-on](https://github.com/reese/ruby-lsp-rubyfmt-formatter). Install the extension:

```bash
# Add to project
bundle add ruby-lsp-rubyfmt-formatter --group development

# Install globally
gem install ruby-lsp-rubyfmt-formatter
```

And then add the following to your `.vscode/settings.json`:

```json
{
  "[ruby]": {
    "editor.defaultFormatter": "Shopify.ruby-lsp",
    "editor.formatOnSave": true
  },
  "rubyLsp.formatter": "rubyfmt"
}
```

Rubyfmt is also supported by the (now-deprecated) [VSCode Ruby extension](https://marketplace.visualstudio.com/items?itemName=rebornix.Ruby). Add the following to your `settings.json`:

```json
{
  "ruby.useLanguageServer": true,
  "ruby.format": "rubyfmt",
  "[ruby]": {
    "editor.formatOnSave": true
  }
}
```

This is additionally supported by the [Formatto for VS Code](https://marketplace.visualstudio.com/items?itemName=damolinx.formatto) extension.  Use it as your project's formatter by adding this entry to your `.vscode/settings.json` after installing the extension: 

```jsonc
{
  "[ruby]": {
    "editor.defaultFormatter": "damolinx.formatto"
  },
}
```

Check its [README](https://github.com/damolinx/vscode-formatto#readme) for additional configuration options.

### Neovim + null-ls

The [null-ls](https://github.com/jose-elias-alvarez/null-ls.nvim) plugin supports rubyfmt out of the box:

```lua
null_ls.setup({
  sources = {
    null_ls.builtins.formatting.rubyfmt,
  },
})
```

See the [null-ls documentation](https://github.com/jose-elias-alvarez/null-ls.nvim/blob/main/doc/BUILTINS.md#rubyfmt) for more details.

### Vim

1. Run `cargo build --release`
2. Add `source /path/to/rubyfmt.vim` to your `~/.vimrc`
3. Add `let g:rubyfmt_path = /path/to/target/release/rubyfmt-main` beneath the source line

### RubyMine (and JetBrains IDEs)

1. Install the [File Watchers plugin](https://www.jetbrains.com/help/ruby/settings-tools-file-watchers.html)
2. Go to File | Settings | Tools | File Watchers
3. Import `watchers.xml` from `editor_plugins/rubymine/`
4. Optionally set Level to Global for all projects

See the [File Watchers documentation](https://www.jetbrains.com/help/ruby/using-file-watchers.html#ws_filewatcher_type_and_location_of_input_files) for more details.

### Sublime Text

Install the [rubyfmt plugin](https://github.com/toreriklinnerud/sublime-rubyfmt/) via Package Control.

Files format on save or with `Cmd + ;` (macOS) / `Alt + ;` (other). Settings:

```json
{
  "ruby_executable": "ruby",
  "rubyfmt_executable": "rubyfmt",
  "format_on_save": true
}
```

### Atom

Install the [rubyfmt package](https://github.com/toreriklinnerud/atom-rubyfmt/) from Settings > Packages.

Files format on save or with `Cmd + ;` (macOS) / `Alt + ;` (other).

## Rubocop

For usage with Rubocop, see the [rubocop-rubyfmt](https://github.com/reese/rubocop-rubyfmt) gem.

## Contributing

Please check out our [contributing guide](./CONTRIBUTING.md).

## Maintenance Status

The original author (fables-tales) is no longer working on open source in their spare time.
Other contributors work regularly on `rubyfmt`, and contributors with commit access are welcome to merge changes that pass CI.
