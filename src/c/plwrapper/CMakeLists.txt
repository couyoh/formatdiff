cmake_minimum_required(VERSION 3.20)
project(plwrapper CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

set(LIBPL_SHARED_LIBRARY ON CACHE BOOL "" FORCE)
set(LIBPL_ENABLE_TESTS OFF CACHE BOOL "" FORCE)
set(LIBPL_ENABLE_CLI OFF CACHE BOOL "" FORCE)
set(LIBPL_BUILD_CLI_AS_EXECUTABLE OFF CACHE BOOL "" FORCE)
set(LIBPL_ENABLE_EXAMPLE OFF CACHE BOOL "" FORCE)
set(LIBPL_ENABLE_PRECOMPILED_HEADERS OFF CACHE BOOL "" FORCE)

add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/../../../vendor/PatternLanguage ${CMAKE_CURRENT_BINARY_DIR}/PatternLanguage)

add_library(plwrapper STATIC plwrapper.cc)
target_include_directories(plwrapper PUBLIC include)
target_link_libraries(plwrapper PUBLIC libpl fmt::fmt-header-only)

install(TARGETS plwrapper ARCHIVE DESTINATION lib)
install(TARGETS libpl LIBRARY DESTINATION lib)
install(FILES include/plwrapper.h DESTINATION include)
