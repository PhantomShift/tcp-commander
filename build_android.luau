#!/bin/lune

local fs = require "@lune/fs"
local process = require "@lune/process"
local stdio = require "@lune/stdio"

local APK_PATH = "src-tauri/gen/android/app/build/outputs/apk"
local BUILD_COMMAND = "cargo"
local BUILD_COMMAND_ARGS = { "tauri", "android", "build", "--apk", "--split-per-abi" }
local DESTINATION_PATH = "target/android_apks"

local result

if not table.find(process.args, "--skip-build") then
    result = process.spawn(BUILD_COMMAND, BUILD_COMMAND_ARGS, {
        stdio = "forward"
    })

    if not result.ok then
        stdio.write(stdio.color("red"))
        print("Build failed...")
        process.exit(1)
    end
end


local apk_folders = fs.readDir(APK_PATH)

if #apk_folders == 0 then
    stdio.write(stdio.color("red"))
    print("[ERROR]: apk folder is empty")
    process.exit(1)
end

fs.writeDir(DESTINATION_PATH)

for _, folder in apk_folders do
    local release_folder = `{APK_PATH}/{folder}/release`
    for _, name in fs.readDir(release_folder) do
        if name:match("%.apk$") then
            local apk_path = `{release_folder}/{name}`
            fs.copy(apk_path, `{DESTINATION_PATH}/{name}`, true)
            process.spawn("sha256sum",
                {`{DESTINATION_PATH}/{name}`},
                {stdio = "forward"}
            )
        end
    end
end
