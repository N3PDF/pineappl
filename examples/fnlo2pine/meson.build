project(
    'fnlo2pine',
    'cpp',
    version : '0.1',
    license : 'GPL3',
    default_options : [ 'cpp_std=c++17', 'warning_level=3' ],
)

applgrid_config = find_program('fnlo-tk-config')

meson.override_dependency('fnlo-tk', declare_dependency(
    compile_args : run_command(applgrid_config, '--cxxflags').stdout().split(),
    link_args : run_command(applgrid_config, '--ldflags').stdout().split(),
    version : run_command(applgrid_config, '--version').stdout(),
))

applgrid_dep = dependency('fnlo-tk', version : '=2.3.1')
pineappl_capi_dep = dependency('pineappl_capi', version : '>=0.3')
lhapdf_dep = dependency('lhapdf')

bin = executable(
    'fnlo2pine',
    'fnlo2pine.cpp',
    dependencies : [ applgrid_dep, lhapdf_dep, pineappl_capi_dep ],
    install : true
)
