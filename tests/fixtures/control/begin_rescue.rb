begin
    b
end
begin
    b
    c
end
begin
end
begin
    # c
end
begin
    b
rescue StandardError
end
begin
rescue StandardError
    c
end
begin
rescue StandardError
end
begin
ensure
end
begin
    b
rescue A, B
end
begin
    b
rescue A
    c
end
begin
    b
rescue A
    c
else
    d
end

begin
    b
ensure
    d
end
begin
    b
ensure
end
begin
    b
rescue StandardError
    # c
end
begin
    b
    # c1
rescue StandardError
    c
    # c2
else
    d
ensure
    e
    # c4
end
begin
    # c
    b
end
begin
    b
rescue A # c
    c
end
begin
    b
end # c
begin
rescue Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
              Bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
              Cccccccc,
              Ddddddddddddddddddddddddddddddddddddddddddddd,
              Eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee # c
    c
end
begin
    b
rescue *A
    c
end
begin
    b
rescue A::B
    c
end
begin
    b
rescue StandardError
    c
end.foo
begin
    b
rescue StandardError
    c
    # c3
ensure
    d
end
begin
    b
    # c
ensure
    d
end
begin
    b
rescue StandardError
    # c2
else
    d
ensure
    e
end
begin
    b
rescue A
    c
    # c2
rescue B
    d
end
begin
    b
rescue A
    c
    # c2
else
    d
end

begin
    # c1
rescue StandardError
end
foo(
    begin
        b
    end,
)
begin
    b
end.foo
(
    begin
        b
    rescue StandardError
        c
    end
).foo
begin
    b
rescue StandardError
    c
end
begin
    foo b
rescue StandardError
    c
end
begin
    b.c(d)
rescue StandardError
    e
end
loop do
    begin
        return a
    rescue StandardError
        b
    end
end
begin
    b
rescue Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
              Bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
    c
end
begin
    b
rescue A, *B
    c
end
