cmake_minimum_required(VERSION 3.24)

project(KineinVectisIcons VERSION 0.1 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.5 REQUIRED COMPONENTS Quick Svg)
qt_standard_project_setup(REQUIRES 6.5)

qt_add_executable(kinein-icons
    main.cpp
)

qt_add_qml_module(kinein-icons
    URI Kinein.Icons
    VERSION 1.0
    QML_FILES
        qml/Main.qml
        qml/IconGallery.qml
        qml/KIcon.qml
    RESOURCES
        qml/IconRegistry.js
        icons/action/ai_context_preview.svg
        icons/action/back.svg
        icons/action/close.svg
        icons/action/command_palette.svg
        icons/action/forward.svg
        icons/action/fullscreen.svg
        icons/action/help.svg
        icons/action/layout.svg
        icons/action/new.svg
        icons/action/notifications.svg
        icons/action/open.svg
        icons/action/open_ai_terminal.svg
        icons/action/refresh.svg
        icons/action/save.svg
        icons/action/save_all.svg
        icons/action/search_everywhere.svg
        icons/action/sync.svg
        icons/action/zen.svg
        icons/build/clean.svg
        icons/build/compile.svg
        icons/build/configure.svg
        icons/build/rebuild.svg
        icons/debug/attach.svg
        icons/debug/breakpoint.svg
        icons/debug/memory.svg
        icons/debug/pause.svg
        icons/debug/registers.svg
        icons/debug/resume.svg
        icons/debug/start.svg
        icons/debug/step_into.svg
        icons/debug/step_out.svg
        icons/debug/step_over.svg
        icons/editor/definition.svg
        icons/editor/format.svg
        icons/editor/modified.svg
        icons/editor/pin.svg
        icons/editor/preview.svg
        icons/editor/quick_fix.svg
        icons/editor/readonly.svg
        icons/editor/references.svg
        icons/editor/rename.svg
        icons/editor/split_down.svg
        icons/editor/split_right.svg
        icons/editor/wrap.svg
        icons/embedded/board.svg
        icons/embedded/cross_compile.svg
        icons/embedded/deploy.svg
        icons/embedded/flash.svg
        icons/embedded/gdbserver.svg
        icons/embedded/local_linux.svg
        icons/embedded/mcu.svg
        icons/embedded/port_forward.svg
        icons/embedded/power.svg
        icons/embedded/probe.svg
        icons/embedded/qemu.svg
        icons/embedded/qemu_display.svg
        icons/embedded/remote_linux.svg
        icons/embedded/remote_run.svg
        icons/embedded/reset.svg
        icons/embedded/sdk.svg
        icons/embedded/serial.svg
        icons/embedded/ssh.svg
        icons/embedded/sysroot.svg
        icons/embedded/target.svg
        icons/file/c.svg
        icons/file/cmake.svg
        icons/file/cpp.svg
        icons/file/device_tree.svg
        icons/file/header.svg
        icons/file/linker_script.svg
        icons/file/qml.svg
        icons/file/rust.svg
        icons/file/toml.svg
        icons/git/blame.svg
        icons/git/branch.svg
        icons/git/commit.svg
        icons/git/conflict.svg
        icons/git/fetch.svg
        icons/git/merge.svg
        icons/git/pull.svg
        icons/git/push.svg
        icons/git/rebase.svg
        icons/git/stage.svg
        icons/git/stash.svg
        icons/git/unstage.svg
        icons/project/collapse_all.svg
        icons/project/follow_active.svg
        icons/project/new_file.svg
        icons/project/new_folder.svg
        icons/quality/profile.svg
        icons/quality/run.svg
        icons/quality/sanitizers.svg
        icons/quality/static_analysis.svg
        icons/run/rerun.svg
        icons/run/start.svg
        icons/run/stop.svg
        icons/search/case_sensitive.svg
        icons/search/regex.svg
        icons/search/replace.svg
        icons/search/whole_word.svg
        icons/status/encoding.svg
        icons/status/line_endings.svg
        icons/status/lock.svg
        icons/status/lsp.svg
        icons/status/toolchain.svg
        icons/test/coverage.svg
        icons/test/run_all.svg
        icons/test/run_nearest.svg
        icons/view/ai_terminal.svg
        icons/view/build.svg
        icons/view/configuration_actions.svg
        icons/view/debug.svg
        icons/view/embedded.svg
        icons/view/git.svg
        icons/view/problems.svg
        icons/view/project.svg
        icons/view/project_health.svg
        icons/view/quality.svg
        icons/view/resources.svg
        icons/view/run.svg
        icons/view/search.svg
        icons/view/services.svg
        icons/view/settings.svg
        icons/view/simulation.svg
        icons/view/targets.svg
        icons/view/tasks.svg
        icons/view/terminal.svg
        icons/view/tests.svg
        icons/view/toolchains.svg
    RESOURCE_PREFIX /kinein
)

target_link_libraries(kinein-icons PRIVATE Qt6::Quick Qt6::Svg)
