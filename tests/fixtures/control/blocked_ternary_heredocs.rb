x =
    if a
        <<~MD
    text
  MD
    else
        ''
    end
a ? '' : foo(<<~MD)
    text
  MD
