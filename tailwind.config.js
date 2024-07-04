/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{html,svelte,js,ts,}",
        "./electron/**/*.{html,svelte,js,ts,}"
    ],
    theme: {
        extend: {},
    },
    plugins: [],
}
// tailwind.config.js
module.exports = {
    content: ['./src/**/*.{html,js,svelte,ts}'],
    darkMode: 'class',
    theme: {
      extend: {
        colors: {
          primary: {
            light: '#ffffff',
            dark: '#000000',
          },
          background: {
            light: '#f8f8f8',
            dark: '#1a1a1a',
          },
          text: {
            light: '#1f2937',
            dark: '#f3f4f6',
          },
        },
        fontFamily: {
          'header': ['Playfair Display', 'serif'],
          'body': ['Inter', 'sans-serif'],
        },
      },
    },
    plugins: [],
  }