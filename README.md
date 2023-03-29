# How do I pronounce `rubyfmt`
* en: Ruby format
* jp: ルビーフォーマット

## How do I use it


### Install from `brew`

On Mac and Linux, `rubyfmt` can be installed with [Homebrew](https://brew.sh/):

```bash
brew install rubyfmt
```

### Build from source

1. Make sure you've got cargo installed
2. Run `make all`
3. Copy target/release/rubyfmt-main to somewhere on your path as `rubyfmt`

Rubyfmt supports the following CLI invocations:

* `<whatever> | rubyfmt` pipe from standard in
* `rubyfmt -i -- files or directories` to format files and directories in place
* `rubyfmt -- files or directories` output rubyfmtted code to STDOUT.
* `rubyfmt -c -- files or directories` output a diff of input and rubyformatted input.
* `rubyfmt --header-opt-in -- files or directories` to format files only with a `# rubyfmt: true` comment at the top of the file
* `rubyfmt --header-opt-out -- files or directories` to skip formatting files with a `# rubyfmt: false` comment at the top of the file

`rubyfmt` also supports ignoring files with a `.rubyfmtignore` file when present in the root of the working directory.
`.rubyfmtignore` uses the same syntax as `.gitignore`, so you can choose to ignore whole directories or use globs as needed.
By default, `rubyfmt` also ignores files in `.gitignore` during file traversal, but you can force these files to be formatted by using the `--include-gitignored` flag.

## Editor Support

### Vim

We aren't currently tested with any vim plugin managers, however, adding the
plugin from a git clone is fairly easy:

* Run `cargo build --release`
* Add `source /path/to/rubyfmt.vim` to your `~/.vimrc` (e.g. [my dotfiles](https://github.com/penelopezone/dotfiles/commit/2c0e9c1215de368e64e063021e9523aa349c5454#diff-2152fa38b4d8bb10c75d6339a959650dR253) please note, this line is commented)
* Add `let g:rubyfmt_path = /path/to/target/release/rubyfmt-main` beneath the source line

### Neovim + LSP + null-ls

If you use the popular [null-ls](https://github.com/jose-elias-alvarez/null-ls.nvim) LSP plugin to manage formatters, it supports `rubyfmt` out of the box. You can add the formatter to your existing `setup()` configuration:

```diff
local null_ls = require("null-ls")

null_ls.setup({
  sources = {
+   null_ls.builtins.formatting.rubyfmt,
  },
})
```

Read more in the [null-ls documentation](https://github.com/jose-elias-alvarez/null-ls.nvim/blob/main/doc/BUILTINS.md#rubyfmt).

### Visual Studio Code

Rubyfmt is a supported formatter in the popular
[vscode ruby extension](https://marketplace.visualstudio.com/items?itemName=rebornix.Ruby).
You should copy `rubyfmt-main` to be called `rubyfmt` on your PATH .
Once installed, add the following to vscode's `settings.json` file:

``` json
  "ruby.useLanguageServer": true,
  "ruby.format": "rubyfmt",
  "[ruby]": {
      "editor.formatOnSave": true
  },
```

### RubyMine (and similar Jetbrains family IDE)

[Install](https://www.jetbrains.com/help/ruby/settings-tools-file-watchers.html) the File Watchers plugin and go to `File | Settings | Tools | File Watchers`. Now import `watchers.xml` from `editor_plugins/rubymine/`. Optionally set `Level` to `Global` to have it available across all projects.

See [this reference](https://www.jetbrains.com/help/ruby/using-file-watchers.html#ws_filewatcher_type_and_location_of_input_files) on using file watchers to learn more.

### Sublime Text

Install the [rubyfmt plugin](https://github.com/toreriklinnerud/sublime-rubyfmt/) from [Package Control](https://packagecontrol.io): Install Package -> rubyfmt.

Ruby files are formatted on save or by pressing `Alt + ;` or on macOS: `Cmd + ;`. `rubyfmt` is assumed to be on path.

Overridable default settings:
 ``` json
 {
   "ruby_executable": "ruby",
   "rubyfmt_executable": "rubyfmt",
   "format_on_save": true,
 }
 ```

 ### Atom

Install the [rubyfmt package](https://github.com/toreriklinnerud/atom-rubyfmt/) from Settings > Packages.

Ruby files are formatted on save or by pressing `Alt + ;` or on macOS: `Cmd + ;` `rubyfmt` is assumed to be on path. See the package settings for more options.

## Contributing

Please checkout [our contributing guide](./CONTRIBUTING.md)
