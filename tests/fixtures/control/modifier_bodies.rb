foo b if a
return if a
return b if a
if a
    b ? c : d
end
if a
    b if c
end
if a
    b unless c
end
b while c if a
b until c if a
(b if c) if a
(b ? c : d) if a
if a
    begin
        b
    rescue StandardError
        c
    end
end
c and d if a
c or d if a
raise b if a
foo { b } if a
foo { b } if a
foo(&b) if a
foo b, &c if a
return 1, 2 if a
"#{x}" if a
{ a: 1 } if a
[1, 2] if a
defined?(x) if a
not b if a
b..c if a
if a
    case b
    when 1
        2
    end
end
c while b if a
if a
    begin
        b
    end
end
if a
    b
    c
end
b and c ? d : e if a
!(b ? c : d) if a
b.c d ? e : f if a
unless a
    b ? c : d
end
unless a
    b if c
end
unless a
    b unless c
end
b unless a
loop do
    break if a
    next if a
    redo if a
end
begin
rescue StandardError
    retry if a
end
if a
    b
else
    d if c
end
if a
    b
else
    c ? d : e
end
if a
    if b
        c
    elsif d
        e
    end
end
