run PublicApi::App.new(
    amqp_client: service.amqp_client,
    factory: service.service_clients.fetch(:factory),
    include_internal_admin_apis: configatron.publicapi2.include_internal_admin_apis,
    service: service,
)

register defaults: :kept,
                  aligned: 'a bare keyword list keeps its aligned continuation style'

mount lambda { |environment| [200, {}, ['ok']] }

raise ArgumentError,
            'a bare argument list keeps its aligned continuation as before'
