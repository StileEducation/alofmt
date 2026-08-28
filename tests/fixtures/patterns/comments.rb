# leading
case foo # trailing
# before in
in 1 # trailing in
    # inside in
    bar
in 2
    # only comment
in 3 if bar # guard comment
else
    # only comment in else
end
case foo
# before first in
# second
in 1
    x

    # after body
in 2
    # empty body comment
in 3
    y
    # col zero before else
else
    z
    # before end in else
end
case foo
in 1
    x
    # before end no else
end
case foo
in 1
    # col zero after empty
in 2
end
case foo # trailing
# blank then comment
in 1
end
case foo
in [
          A, # first
          # own line
          B
      ]
    x
in Integer | String # or
    w
end
