a ? foo(b) : d
a ? b && c : d
if a
    b and c
else
    d
end
a ? !b : d
a ? not(b) : d
a ? -b : d
if a
    raise b
else
    d
end
a ? raise(b) : d
a ? b.c : d
if a
    b.c d
else
    d
end
a ? b.c(d) : d
a ? b[c] : d
a ? 's' : d
a ? [1] : d
a ? nil : d
if a
    b ? c : e
else
    d
end
if a
    b if c
else
    d
end
a ? @b : d
a ? B : d
a ? C::D : d
a ? b { c } : d
a ? b(&c) : d
a ? b..c : d
if a
    defined?(b)
else
    d
end
a ? b + c : d
a ? b == c : d
a ? "#{b}" : d
if a
    b unless c
else
    d
end
a ? (b if c) : d
a ? (b) : d
if a
    (
        b
        c
    )
else
    d
end
a ? b.c { d } : d
a ? b! : d
a ? b.c&.d : d
a ? ::B : d
a ? b**c : d
a ? b.c.d.e : d
if a
    b while c
else
    d
end
if a
    begin
        b
    rescue StandardError
        c
    end
else
    d
end
if a
    foo b do
        c
    end
else
    d
end
if a
    b or c
else
    d
end
if a
    b == c ? d : e
else
    d
end
a ? __method__ : d
loop do
    if a
        break
    else
        next
    end
    if a
        break b
    else
        next c
    end
end
a ? not(b) : not(c)
a ? not(b) : c
