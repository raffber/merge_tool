include(FetchContent)

if(CMAKE_HOST_SYSTEM_NAME STREQUAL Linux)
    FetchContent_Declare(
        merge_tool
        DOWNLOAD_NO_EXTRACT TRUE
        URL https://github.com/raffber/merge_tool/releases/download/release%2F0.2.3/merge_tool
    )

    FetchContent_Populate(merge_tool)

    set(MERGE_TOOL_BIN ${merge_tool_SOURCE_DIR}/merge_tool)
    execute_process(COMMAND "chmod +x ${MERGE_TOOL_BIN}")
elseif(CMAKE_HOST_SYSTEM_NAME STREQUAL Windows)
    FetchContent_Declare(
        merge_tool
        DOWNLOAD_NO_EXTRACT TRUE
        URL https://github.com/raffber/merge_tool/releases/download/release%2F0.2.3/merge_tool.exe
    )
    FetchContent_Populate(merge_tool)

    set(MERGE_TOOL_BIN ${merge_tool_SOURCE_DIR}/merge_tool.exe)
endif()

execute_process(
  COMMAND ${MERGE_TOOL_BIN} --version
  OUTPUT_VARIABLE MERGE_TOOL_VERSION
  OUTPUT_STRIP_TRAILING_WHITESPACE)

message("Using merge_tool ${MERGE_TOOL_VERSION}")


function(merge_tool_generate ARGS)
    set(options OPTIONAL USE_BACKDOOR)
    set(oneValueArgs TARGET CONFIG_FILE APP BTL)

    cmake_parse_arguments(ARGS "${options}" "${oneValueArgs}" "" ${ARGN})

    set(BACKDOOR_FLAG "")
    if (USE_BACKDOOR)
        set(BACKDOOR_FLAG "--use-backdoor")
    endif()

    add_custom_target(
      ${TARGET} ALL
      COMMAND "${MERGE_TOOL_BIN}" generate ${BACKDOOR_FLAG} -c "${CONFIG_FILE}" -o "${CMAKE_CURRENT_BINARY_DIR}" --repo-path "${CMAKE_CURRENT_SOURCE_DIR}"
      DEPENDS ${APP_HEX} ${BTL_HEX}
      BYPRODUCTS ${CMAKE_CURRENT_BINARY_DIR}/info.json)
endfunction()


function(merge_tool_bundle ARGS)
    set(options VERSIONED)
    set(oneValueArgs TARGET INFO_FILE OUTPUT_DIR)

    cmake_parse_arguments(ARGS "${options}" "${oneValueArgs}" "" ${ARGN})

    set(VERSIONED_FLAG "")
    if (VERSIONED)
        set(VERSIONED_FLAG "--versioned")
    endif()

    add_custom_target(
      ${TARGET} ALL
      COMMAND "${MERGE_TOOL_BIN}" bundle -i ${INFO_FILE} -o ${OUTPUT_DIR} ${VERSIONED_FLAG}
      DEPENDS ${INFO_FILE})
endfunction()