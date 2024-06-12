function(check_kwui_target)
    if (NOT TARGET kwui)
        add_library(kwui SHARED IMPORTED)
        if (CMAKE_BUILD_TYPE STREQUAL "Debug")
            set_target_properties(kwui PROPERTIES
                IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/target/aarch64-linux-android/debug/libkwui.so)
        else()
            set_target_properties(kwui PROPERTIES
                IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/target/aarch64-linux-android/release/libkwui.so)
        endif()
    endif()
endfunction()


function(make_rust_apk_ndk_library TARGET)
    check_kwui_target()

    if(NOT TARGET ${TARGET})
        add_library(${TARGET} INTERFACE)
    endif()
    make_apk_ndk_library(${TARGET})
    set(ANDROIDPACKAGING_LIB_ROOT "${CMAKE_CURRENT_BINARY_DIR}/${TARGET}_apk/app/src/main/jniLibs/${CMAKE_ANDROID_ARCH_ABI}/")
    add_custom_command(TARGET ${TARGET}.APK PRE_BUILD
        # COMMENT "Build native rust library..."
        DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/target/${TARGET}.cargo.stamp
        COMMAND cmake -E echo "...Build native rust library..." &&
            cargo build --target aarch64-linux-android -p ${TARGET} --lib &&
            cmake -E echo "Copying ${CMAKE_CURRENT_SOURCE_DIR}/target/aarch64-linux-android/debug/lib${TARGET}.so to ${ANDROIDPACKAGING_LIB_ROOT}" &&
            cmake -E copy ${CMAKE_CURRENT_SOURCE_DIR}/target/aarch64-linux-android/debug/lib${TARGET}.so ${ANDROIDPACKAGING_LIB_ROOT} &&
            cmake -E touch ${CMAKE_CURRENT_SOURCE_DIR}/target/${TARGET}.cargo.stamp
        VERBATIM
    )
endfunction()