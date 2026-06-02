if exists('g:loaded_rubyfmt')
  finish
endif
let g:loaded_rubyfmt = 1

let s:cpo_save = &cpo
set cpo&vim

" Path to the rubyfmt binary. Defaults to whatever is on $PATH; override in
" your vimrc, e.g. let g:rubyfmt_path = '/path/to/target/release/rubyfmt-main'
if !exists('g:rubyfmt_path')
  let g:rubyfmt_path = 'rubyfmt'
endif

if !exists("s:rubyfmt_ac_set")
  let s:rubyfmt_ac_set=1
  autocmd FileType ruby autocmd! BufWritePre <buffer> call rubyfmt#format()
endif

let &cpo = s:cpo_save
unlet s:cpo_save
