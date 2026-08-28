case a
when 1
    b
end
case a
when 1
    b
end
case a
when 1, 2
    b
when 3
    c
else
    d
end
case a
when 1
end
case a
when 1
else
end
case a
when 1
    # c
end
case a
when 1
    b
    # c
when 2
    c
end
case a
when 1
    b
    # c
when 2
    c
end
case a # c
when 1
    b
end
case a
# c
when 1
    b
end
case a
when 1 # c
    b
end
case a
when 1,
          2 # c
    b
end
case a
when 1
    b
else
    c
    # c
end
case a
when 1
    b
    # c
else
    c
end
case a
when 1
    b
end # c
case
when a
    b
when c
    d
end
case
when a
end
case a
when *b
    c
end
case a
when 1, *b
    c
end
case a
when Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
          Bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, Cccccccc
    c
end
case a
when Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
          Bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, Cccccccc,
          Ddddddddddddddddddddddddddddddddddddddddddddd,
          Eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
    c
end
case a
when Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
          Bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
          Cccccccc,
          Ddddddddddddddddddddddddddddddddddddddddddddd,
          Eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee # c
    c
end
case a
when Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
          Bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, Cccccccc, # c
          Ddddddddddddddddddddddddddddddddddddddddddddd,
          Eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
    c
end
case a
when Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
          Bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, foo(
              Cccccccc,
              Ddddddddddddddddddddddddddddddddddddddddddddd,
          ), Eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
    c
end
case a
when 1,
          2,
          foo(3) # c
    b
end
case a
when 1, 2, <<~EOS, 3
  x
EOS
    b
end
case aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.bbbbbbbbbbbbbbbbbbbbbbbbbbbb
when 1
    b
end
case foo(
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
)
when 1
    c
end
case aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa &&
    bbbbbbbbbbbbbbbbbbbbbb
when 1
    c
end
foo(
    case a
    when 1
        b
    end,
)
case a
when 1
    b
end.foo
case a
when 1
    b
    c
end
case a
when 1
    b if c
when 2
    d ? e : f
end
case a
when 1
    b
when 2
    c
end
case a
when foo(
              aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
              bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
          )
    c
end
case a
when 1
    b
else
    c
end
case a
when 1..2, 3
    b
end
case a
when 1, 2
    b # c
end
case a
when 1, 2, foo do
              a
              b
          end
    b
end
