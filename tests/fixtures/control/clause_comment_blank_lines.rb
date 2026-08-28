case a
when 1
    raise

    # c1
    # c2
when 2
    raise
    # c3
when 3
    raise
    # c4
else
    raise

    # c5
end
begin
    a

    # c6
rescue B
    b

    # c7
else
    c
ensure
    d

    # c9
end
if a
    b

    # c10
elsif c
    d

    # c11
else
    e

    # c12
end
while a
    b

    # c13
end
def foo
    a

    # c14
end
foo do
    a

    # c15
end
case a
when 1
    # c16
    b
end
case a
when 1
    b

    # c17

    # c18
when 2
    b
end
