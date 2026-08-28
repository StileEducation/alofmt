undef foo
undef foo, bar
undef foo, :'bar', bar=
undef :"foo#{1}"
undef `
undef foo # trailing
alias foo bar
alias foo bar
alias $a $b
alias foo bar
alias :'foo' bar
alias :"foo#{1}" bar
alias == eql?
alias [] fetch
alias foo bar # trailing
# leading
alias foo bar
