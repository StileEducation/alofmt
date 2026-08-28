x = <<~EOS
  hello
EOS
x = <<~EOS.strip
  hello
EOS
x ||= <<~EOS
  hello
EOS
x = <<~EOS.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
  hi
EOS
x = <<~EOS.aaaaaaaaaaaaaaaaaaaaaaa(
  hi
EOS
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccccccccc,
)
x = foo(<<~EOS)
  hi
EOS
x =
    foo(
        <<~EOS,
  hi
EOS
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
    )
x = <<~EOS # trailing
  hi
EOS
foo[1] ||= <<~EOS
  hi
EOS
x = y = <<~EOS
  hi
EOS
