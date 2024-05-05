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

    # TODO: build libkwui.so
    add_library(kwui SHARED IMPORTED)
    if (CMAKE_BUILD_TYPE STREQUAL "Debug")
        set_target_properties(kwui PROPERTIES
            IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/target/aarch64-linux-android/debug/libkwui.so)
    else()
        set_target_properties(kwui PROPERTIES
            IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/target/aarch64-linux-android/release/libkwui.so)
    endif()
  
    # set_target_properties(
    #     imported::target
    #     PROPERTIES
    #     IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/target/aarch64-linux-android/debug/libinstaller.so
    # )

    add_custom_target(
        installer ALL
        DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/target/aarch64-linux-android/debug/libinstaller.so
        COMMAND cargo build --target aarch64-linux-android -p installer --lib
    )


    # add_library(installer INTERFACE)
    # set_target_properties(installer PROPERTIES
    #     IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/target/aarch64-linux-android/debug/libinstaller.so)
    # add_custom_command(TARGET installer PRE_BUILD
    #     COMMAND cargo build --target aarch64-linux-android -p installer --lib
    #     COMMENT "Building native lib"
    # )
    make_apk_ndk_library(installer)
endif()
