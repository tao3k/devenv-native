using Pkg

function dependency_source(name::AbstractString)::String
    matches = filter(dependency -> dependency.name == name, collect(values(Pkg.dependencies())))
    isempty(matches) && error("Dependency $(name) is not available in the active project")
    length(matches) == 1 || error("Expected one dependency named $(name), found $(length(matches))")
    source = only(matches).source
    isnothing(source) && error("Dependency $(name) does not expose a source path")
    return source
end

function parser_library_suffix()
    Sys.islinux() && return ".so"
    Sys.isapple() && return ".dylib"
    Sys.iswindows() && return ".dll"
    error("Unsupported platform for OMParser source build")
end

function locate_parser_library(root::AbstractString)
    isdir(root) || return nothing
    suffix = parser_library_suffix()
    for (directory, _, files) in walkdir(root)
        for file in files
            if occursin("libomparse-julia", file) && endswith(file, suffix)
                return joinpath(directory, file)
            end
        end
    end
    return nothing
end

function load_path_with_stdlib()
    load_path = get(ENV, "JULIA_LOAD_PATH", "")
    isempty(load_path) && return "@:@stdlib"
    occursin("@stdlib", load_path) && return load_path
    return string(load_path, endswith(load_path, ":") ? "" : ":", "@stdlib")
end

function force_omparser_source_build!()
    Pkg.instantiate()
    package_root = dependency_source("OMParser")
    parser_root = joinpath(package_root, "lib", "parser")
    build_root = joinpath(package_root, "lib", "build")
    build_lib_root = joinpath(build_root, "lib")
    isdir(parser_root) || error("OMParser parser source directory does not exist: $(parser_root)")

    rm(build_root; recursive = true, force = true)
    cd(parser_root) do
        run(`autoconf`)
        withenv("JULIA_LOAD_PATH" => load_path_with_stdlib()) do
            run(`./configure`)
            run(`make`)
        end
    end

    library_path = locate_parser_library(build_lib_root)
    isnothing(library_path) &&
        error("OMParser source build completed without libomparse-julia under $(build_lib_root)")
    @info "OMParser source build ready" library_path
end

force_omparser_source_build!()
