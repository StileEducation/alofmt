case a
when 1
    b
else # c
    c
end
if a
    b
else # c6
    c
end
begin
    b
rescue StandardError # c
    c
end
begin
    b
ensure # c
    c
end
begin
    b
rescue A
    z
else # c
    c
end
