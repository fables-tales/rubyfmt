# How do I pronounce `rubyfmt`
Ruby format

## How do I use rubyfmt

1. clone this repo
2. `make`
3. `make install`
4. put this snippet in your vimrc, or if you're using a different editor, IDK
run `ruby --disable=gems ~/bin/rubyfmt.rb -i <the_current_file>` on save.

```
function Rubyfmt()
  echo "Calling rubyfmt"
  silent exec("!ruby --disable=gems ~/bin/rubyfmt.rb -i " . expand("%"))
  silent exec("edit " . expand("%"))
endfunction

au BufWritePost *.rb call Rubyfmt()
```

# We're doing dev on master please check out the v0.2 tag
# We're doing dev on master please check out the v0.2 tag
# We're doing dev on master please check out the v0.2 tag
