<<~EOS
  hello
    there
  #{@x}
EOS
<<-EOS
  hello
  EOS
<<EOS
hello
EOS
<<~'EOS'
  hello #{x}
EOS
<<~"EOS"
  hello #{@x}
EOS
<<~`EOS`
  ls
EOS
<<~`EOS`
  ls #{@x}
EOS
<<~EOS

  a

  b

EOS
<<~EOS
EOS
<<~"EOS"
  #{@x}
EOS
<<~EOS
  a \
  b
EOS
<<~EOS
  a #{@y} b #{@x}
  #{@z}
EOS
<<~EOS # comment
  a
EOS
# leading
<<~EOS
  a
EOS
<<~EOS
  a
EOS
<<~EOS
  b
EOS

<<~EOS
  c
EOS
<<~EOS
  trailing   
  whitespace	
EOS
<<~'EOS'
  it's "quoted" \n
EOS
