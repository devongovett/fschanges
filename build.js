const os = require("os");
const { execSync } = require("child_process");

// windows, mac and modern linux
execSync("npm run prebuild:current");

// Run the docker builds on os x and linux
if (os.platform() === "darwin" || os.platform() === "linux") {
  // centos (old libc) and alpine
  execSync("npm run prebuild:docker");
}