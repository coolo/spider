cmake_minimum_required(VERSION 3.1.0)

project(spider VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt5 COMPONENTS Core REQUIRED)

add_executable(spider
    main.cpp
    card.cpp
    SpookyV2.cpp
)

target_link_libraries(spider Qt5::Core)
