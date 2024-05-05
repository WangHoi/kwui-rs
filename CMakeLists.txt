cmake_minimum_required(VERSION 3.20)

# Allow the user to specify the MSVC runtime
if(POLICY CMP0091)
  cmake_policy(SET CMP0091 NEW)
endif(POLICY CMP0091)

project(kwui-rs)

if(ANDROID)
    set(KWUI_SYS_DIR ${CMAKE_CURRENT_SOURCE_DIR}/kwui-sys/deps/kwui)
    include(${KWUI_SYS_DIR}/cmake/AndroidPackaging-Prerequisites.cmake)
    include(${KWUI_SYS_DIR}/cmake/AndroidPackaging.cmake)

    include(${CMAKE_CURRENT_SOURCE_DIR}/cmake/AndroidPackaging.cmake)

    make_rust_apk_ndk_library(installer)
    make_rust_apk_ndk_library(rss_reader)
endif()
