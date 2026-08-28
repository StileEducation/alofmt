begin
    a
rescue Atlas::NotFound, Atlas::PermissionDenied # Authorization failures remain distinct from missing records
    b
rescue A # c
    b
rescue A => e # c
    b
rescue Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
              Bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
              Cccccccccccccccc # c
    b
end
