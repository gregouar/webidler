/** @type {import('tailwindcss').Config} */
const plugin = require('tailwindcss/plugin')

module.exports = {
    content: {
      files: ["*.html", "./src/**/*.rs"],
      transform: {
        rs: (content) => content.replace(/(?:^|\s)class:/g, ' '),
      },
    },
    safelist: [
      "duration-100",
      "duration-1000",
      {
        pattern: /bg-\[url\(.*\)\]/,
      },
    ],
    theme: {
      extend: {
        textShadow: {
          sm: '0 1px 2px var(--tw-shadow-color)',
          DEFAULT: '0 2px 4px var(--tw-shadow-color)',
          lg: '0 8px 16px var(--tw-shadow-color)',
        },
        backgroundImage: {
          'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
        }
      },
    },
    plugins: [
      plugin(function ({ matchUtilities, theme }) {
        matchUtilities(
          {
            'text-shadow': (value) => ({
              textShadow: value,
            }),
          },
          { values: theme('textShadow') }
        )
      }),
    ],
  }
