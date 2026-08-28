begin
    a
rescue => e
    bar

    # blank line kept before a clause body's last comment
end
case x
when A
    raise

    # All other
when B
    raise
    # directly before the clause
else
    raise

    # before end
end
case x
in A
    y

    # c
in B
    y
end
