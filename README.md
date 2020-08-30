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

## I want to help!

Check out our [contributing guide](./CONTRIBUTING.md)

## Useful environment variables:

* `RUBYFMT_DISABLE_SZUSH=1`: disables the backend render queue writer,
  very useful for debugging, literally useless if you're not developing rubyfmt
  itself.

## Editor Support

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
