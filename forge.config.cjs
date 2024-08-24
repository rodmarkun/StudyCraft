const path = require('path');

module.exports = {
  packagerConfig: {
    asar: true,
    icon: path.join(__dirname, 'src', 'assets', 'StudyCraftLogo') // no file extension
  },
  rebuildConfig: {},
  makers: [
    {
      name: '@electron-forge/maker-squirrel',
      config: {
        iconUrl: 'https://url/to/icon.ico', // Replace with a URL to your icon
        setupIcon: path.join(__dirname, 'src', 'assets', 'StudyCraftLogo.ico')
      },
    },
    {
      name: '@electron-forge/maker-zip',
      platforms: ['darwin'],
    },
    {
      name: '@electron-forge/maker-deb',
      config: {
        options: {
          icon: path.join(__dirname, 'src', 'assets', 'StudyCraftLogo.png')
        }
      },
    },
    {
      name: '@electron-forge/maker-rpm',
      config: {
        options: {
          icon: path.join(__dirname, 'src', 'assets', 'StudyCraftLogo.png')
        }
      },
    },
  ],
  plugins: [
    {
      name: '@electron-forge/plugin-vite',
      config: {
        build: [
          {
            entry: 'electron/main.cjs',
            config: 'vite.config.js',
          },
        ],
        renderer: [
          {
            name: 'main_window',
            config: 'vite.config.js',
          },
        ],
      },
    },
  ],
};