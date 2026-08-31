BACKEND_CBD_SERVICE_DIRS = Dir[File.dirname(__FILE__) + '/*/Rakefile']
    .map do |file|
        %r{.*/(?<service_dir>[^/]+)/Rakefile$}.match(file)['service_dir']
    end
    .without('public-api-test')
    .sort

stdout, status = Open3.capture2(
    'git',
    'diff',
    '--name-only',
    'origin/master..HEAD',
    dir,
)

SERVICE_DIRS_TO_BUILD = BACKEND_CBD_SERVICE_DIRS.without(
    'lib-stile-ruby',
).without('export-service')

a_value_without_an_internal_break =
    :a_symbol_that_is_far_too_long_for_the_line_to_hold

a_bare_command_value =
    puts_like_command 'one argument', 'and another one', 'and a third'
