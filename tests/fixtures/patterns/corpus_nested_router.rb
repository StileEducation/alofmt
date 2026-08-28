application do
    middleware do
        router do
            transaction do
                case [method, path]
                in ['GET', '/health']
                    health
                in ['POST', %r{\A/books/(?<id>[^/]+)/loans\z} => match]
                    create_loan(match[:id], body)
                in [
                          'PATCH',
                          %r{\A/libraries/(?<library>[^/]+)/books/(?<book>[^/]+)\z} => match
                      ]
                    update_book(match[:library], match[:book], body)
                in [
                          'GET',
                          %r{\A/libraries/(?<library>[^/]+)/shelves/(?<shelf>\d+)/books\z} =>
                              match
                      ]
                    list_books(match[:library], match[:shelf].to_i)
                in ['DELETE', %r{\A/books/(?<id>[^/]+)\z} => match]
                    delete_book(match[:id])
                in ['GET', '/search', { query:, limit: }] if limit <= 100
                    search(query, limit)
                in ['PUT', '/preferences', { theme: 'light' | 'dark' => theme }]
                    update_theme(theme)
                in ['POST', '/events', [String => name, *attributes]]
                    record_event(name, attributes)
                else
                    not_found(method, path)
                end
            end
        end
    end
end
