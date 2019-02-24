# Rubyfmt

```
!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

           Please only submit pull requests if they make %w arrays
           or heredocs work, I'm work in progress on nearly everything
           else
           
!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
````

** VERY MUCH STILL A WORK IN PROGRESS **

Rubyfmt is an autoformatter for Ruby that deals with all the layout of your
ruby code for you. It's inspired by [gofmt](https://golang.org/cmd/gofmt/) in
that it wants to be in your editor's after save hook, and just run every time
you save your file. This means that it should be *fast*. Currently it seems to
run in close to 50ms on even large ruby files. It's also inspired by
[standard](https://github.com/testdouble/standard). Rubyfmt has no configuration
options as to how it actually formats your files. Rather, we're aiming to achieve
an output format that follows well established Ruby community norms.

## Install
Rubyfmt isn't a gem, but rather designed to be run as a single, standalone, ruby
script. Right now, you can clone this repo and then copy `src/rubyfmt.rb` in to your
path.

## Usage

`rubyfmt file.rb` will produce a formatted version of the file on stdout.

## Examples

### Straw man example
``` ruby
require("net/http")
require("pp")

module Cat
  module Dog
    class HTTPClient
      def initialize(target_url)
        @target_url = target_url
      end

      # @public
      def call
        uri = URI(target_url)
        Net::HTTP.get(uri)
      end

      private

      attr_reader(:target_url)
    end
  end
end

p(Cat::Dog::HTTPClient.new("http://example.com/").call)
```
