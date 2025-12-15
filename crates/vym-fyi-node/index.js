const { join } = require('path')
const { platform, arch } = process

// Map platform and architecture to the correct binary name
const platformMap = {
  linux: {
    x64: 'linux-x64-gnu',
    arm64: 'linux-arm64-gnu',
  },
  darwin: {
    x64: 'darwin-x64',
    arm64: 'darwin-arm64',
  },
  win32: {
    x64: 'win32-x64-msvc',
    ia32: 'win32-ia32-msvc',
    arm64: 'win32-arm64-msvc',
  },
}

const platformId = platformMap[platform]?.[arch]
if (!platformId) {
  throw new Error(`Unsupported platform: ${platform} ${arch}`)
}

// Try to load the native module
try {
  module.exports = require(`./vym-fyi-node.${platformId}.node`)
} catch (err) {
  throw new Error(`Failed to load native module: ${err.message}`)
}
