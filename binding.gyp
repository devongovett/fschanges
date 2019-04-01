{
  "targets": [
    {
      "target_name": "fschanges",
      "defines": [ "NAPI_DISABLE_CPP_EXCEPTIONS" ],
      "sources": [ "src/FSChanges.cc", "src/Watcher.cc", "src/Backend.cc", "src/DirTree.cc" ],
      "include_dirs" : ["<!@(node -p \"require('node-addon-api').include\")"],
      "dependencies": ["<!(node -p \"require('node-addon-api').gyp\")"],
      'cflags!': [ '-fno-exceptions' ],
      'cflags_cc!': [ '-fno-exceptions' ],
      "conditions": [
        ['OS=="mac"', {
          "sources": [
            "src/watchman/BSER.cc",
            "src/watchman/watchman.cc",
            "src/shared/BruteForceBackend.cc",
            "src/unix/fts.cc",
            "src/macos/FSEvents.cc"
          ],
          "link_settings": {
            "libraries": ["CoreServices.framework"]
          },
          "defines": [
            "WATCHMAN",
            "BRUTE_FORCE",
            "FS_EVENTS"
          ],
          "xcode_settings": {
            "GCC_ENABLE_CPP_EXCEPTIONS": "YES"
          }
        }],
        ['OS=="linux"', {
          "sources": [
            "src/watchman/BSER.cc",
            "src/watchman/watchman.cc",
            "src/shared/BruteForceBackend.cc",
            "src/linux/InotifyBackend.cc",
            "src/unix/fts.cc"
          ],
          "defines": [
            "WATCHMAN",
            "INOTIFY",
            "BRUTE_FORCE"
          ]
        }],
        ['OS=="win"', {
          "sources": [
            "src/watchman/BSER.cc",
            "src/watchman/watchman.cc",
            "src/shared/BruteForceBackend.cc",
            "src/windows/WindowsBackend.cc"
          ],
          "defines": [
            "WATCHMAN",
            "WINDOWS",
            "BRUTE_FORCE"
          ],
          "msvs_settings": {
            "VCCLCompilerTool": {
              "ExceptionHandling": 1,  # /EHsc
            }
          }
        }]
      ]
    }
  ]
}
