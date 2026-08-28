y =
    "#{
        # c
        foo
    }"
y =
    "a #{
        # c
        foo; bar
    } b"
z = <<~EOS
  a
  #{
    # c
    b
}
  c #{d}
EOS
foo(
    <<~EOS,
  #{
        # c
        b
    }
EOS
    x,
)
# after
