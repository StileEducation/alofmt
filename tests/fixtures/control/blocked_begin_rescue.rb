begin
    b
rescue A
    c
rescue B, C => e
    d
rescue => e
    f
ensure
    g
end
begin
    b
rescue A => e
end
begin
    b
rescue A, B => e # c
    c
end
begin
    b
rescue Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
              Bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb => e
    c
end
begin
    b
rescue Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
              Bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
              Ccccccccccccccccc => e
    c
end
begin
    b
rescue A, *B => e
    c
end
begin
    b
rescue => @e
    c
end
