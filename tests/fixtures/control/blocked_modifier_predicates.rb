if x = foo(b)
    b
end
if x ||= b
    b
end
if @a = b
    b
end
if (a = b)
    b
end
if a[1] = 2
    b
end
if a.b = 2
    b
end
if A = b
    b
end
if $a = b
    b
end
if a and b = c
    b
end
if a = b and c
    b
end
if !(a = b)
    b
end
if foo(a = b)
    b
end
if [a = b]
    b
end
if (a = b) && c
    b
end
if a || b = c
    b
end
if a ? b = c : d
    b
end
if a.b { c = d }
    b
end
if a == (b = c)
    b
end
