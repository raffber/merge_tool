include(FetchContent)

if(CMAKE_HOST_SYSTEM_NAME STREQUAL Linux)
    FetchContent_Declare(
        merge_tool
        DOWNLOAD_NO_EXTRACT TRUE
        URL https://github.com/raffber/merge_tool/releases/download/release%2F0.3.0-alpha.4/merge_tool
    )

    FetchContent_Populate(merge_tool)

    file(COPY ${merge_tool_SOURCE_DIR}/merge_tool DESTINATION ${merge_tool_BINARY_DIR} FILE_PERMISSIONS OWNER_READ OWNER_WRITE OWNER_EXECUTE GROUP_READ GROUP_EXECUTE WORLD_READ WORLD_EXECUTE)

    set(MERGE_TOOL_BIN ${merge_tool_BINARY_DIR}/merge_tool)

elseif(CMAKE_HOST_SYSTEM_NAME STREQUAL Windows)
    FetchContent_Declare(
        merge_tool
        DOWNLOAD_NO_EXTRACT TRUE
        URL https://github.com/raffber/merge_tool/releases/download/release%2F0.3.0-alpha.4/merge_tool.exe
    )

    FetchContent_Populate(merge_tool)

    set(MERGE_TOOL_BIN ${merge_tool_SOURCE_DIR}/merge_tool.exe)
endif()

execute_process(
    COMMAND ${MERGE_TOOL_BIN} --version
    OUTPUT_VARIABLE MERGE_TOOL_VERSION
    RESULT_VARIABLE MERGE_TOOL_RESULT
    OUTPUT_STRIP_TRAILING_WHITESPACE)

if(NOT MERGE_TOOL_RESULT EQUAL 0)
    message(FATAL_ERROR "Failed to execute merge_tool: ${MERGE_TOOL_RESULT}")
endif()

message("Using merge_tool ${MERGE_TOOL_VERSION}")

function(merge_tool_generate)
    set(options USE_BACKDOOR)
    set(oneValueArgs TARGET_NAME CONFIG_FILE APP BTL)

    cmake_parse_arguments(ARG "${options}" "${oneValueArgs}" "" ${ARGN})

    set(BACKDOOR_FLAG "")

    if(ARG_USE_BACKDOOR)
        set(BACKDOOR_FLAG "--use-backdoor")
    endif()

    add_custom_target(${ARG_TARGET_NAME} ALL
        BYPRODUCTS ${CMAKE_CURRENT_BINARY_DIR}/info.json
        DEPENDS ${ARG_APP} ${ARG_BTL}
        COMMAND "${MERGE_TOOL_BIN}" generate ${BACKDOOR_FLAG} -c "${ARG_CONFIG_FILE}" -o "${CMAKE_CURRENT_BINARY_DIR}" --repo-path "${CMAKE_CURRENT_SOURCE_DIR}"
    )
endfunction()

function(merge_tool_bundle)
    set(options VERSIONED)
    set(oneValueArgs TARGET_NAME INFO_FILE OUTPUT_DIR)

    cmake_parse_arguments(ARG "${options}" "${oneValueArgs}" "" ${ARGN})

    set(VERSIONED_FLAG "")

    if(ARG_VERSIONED)
        set(VERSIONED_FLAG "--versioned")
    endif()

    add_custom_target(${ARG_TARGET_NAME} ALL
        COMMAND ${CMAKE_COMMAND} -E rm -rf ${ARG_OUTPUT_DIR}
        COMMAND "${MERGE_TOOL_BIN}" bundle -i ${ARG_INFO_FILE} --output-dir ${ARG_OUTPUT_DIR} ${VERSIONED_FLAG}
        DEPENDS ${ARG_INFO_FILE})
endfunction()
