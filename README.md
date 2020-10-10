# How do I pronounce `rubyfmt`
* en: Ruby format
* jp: ルビーフォーマット

## Does it work right now?

No, but it's getting really close

## How do I use it

Build it:
1. Make sure you've got cargo installed
2. Run `make all`
3. Copy target/release/rubyfmt-main to somewhere on your path as `rubyfmt`

Rubyfmt supports the following CLI invocations:

* `<whatever> | rubyfmt` pipe from standard in
* `rubyfmt filename` to format a file to
  stdout
* `rubyfmt -i files or directories` format files and directories in place
* `rubyfmt directory` to format all ruby files in that directory in place

## Useful environment variables:

* `RUBYFMT_DISABLE_SZUSH=1`: disables the backend render queue writer,
  very useful for debugging, literally useless if you're not developing rubyfmt
  itself.

## Editor Support

### Vim

We aren't currently tested with any vim plugin managers, however, adding the
plugin from a git clone is fairly easy:

* Run `cargo build --release`
* Add `source /path/to/rubyfmt.vim` to your `~/.vimrc` (e.g. [my dotfiles](https://github.com/penelopezone/dotfiles/commit/2c0e9c1215de368e64e063021e9523aa349c5454#diff-2152fa38b4d8bb10c75d6339a959650dR253) please note, this line is commented)
* Add `let g:rubyfmt_path = /path/to/target/release/rubyfmt-main` beneath the source line

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

[Install](https://www.jetbrains.com/help/ruby/settings-tools-file-watchers.html) the File Watchers plugin and configue it like shown below.

<img src="https://user-images.githubusercontent.com/8165/90933920-3b32eb80-e3b5-11ea-9a38-120249d022a3.png" height="300" />

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
