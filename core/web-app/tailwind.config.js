/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx}',
    './src/components/**/*.{js,ts,jsx,tsx}',
    './src/app/**/*.{js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {
      backgroundColor: {
        'red-50': '#fef2f2',
        'red-75': '#fee2e2',
        'red-100': '#fecaca',
        'red-200': '#fca5a5',
        'red-400': '#f87171',
        'red-600': '#ef4444',
        'red-800': '#dc2626',
        'red-1000': '#b91c1c',

        'gray-50': '#fafafb',
        'gray-75': '#f5f5f7',
        'gray-100': '#f4f4f5',
        'gray-200': '#e5e6e9',
        'gray-400': '#b9bac1',
        'gray-600': '#7f8083',
        'gray-800': '#4b4c4f',
        'gray-1000': '#1a1b1d',

        'blue-50': '#e4f1ff',
        'blue-75': '#d5e7ff',
        'blue-100': '#cad3fd',
        'blue-200': '#b2bdfd',
        'blue-400': '#3354ff',
        'blue-600': '#2d3f9a',
        'blue-800': '#1d2963',
        'blue-1000': '#151d4b',
      },
      textColor: {
        'red-50': '#fef2f2',
        'red-75': '#fee2e2',
        'red-100': '#fecaca',
        'red-200': '#fca5a5',
        'red-400': '#f87171',
        'red-600': '#ef4444',
        'red-800': '#dc2626',
        'red-1000': '#b91c1c',

        'gray-50': '#fafafb',
        'gray-75': '#f5f5f7',
        'gray-100': '#f4f4f5',
        'gray-200': '#e5e6e9',
        'gray-400': '#b9bac1',
        'gray-600': '#7f8083',
        'gray-800': '#4b4c4f',
        'gray-1000': '#1a1b1d',

        'blue-50': '#e4f1ff',
        'blue-75': '#d5e7ff',
        'blue-100': '#cad3fd',
        'blue-200': '#b2bdfd',
        'blue-400': '#3354ff',
        'blue-600': '#2d3f9a',
        'blue-800': '#1d2963',
        'blue-1000': '#151d4b',
      },
      borderColor: {
        'red-50': '#fef2f2',
        'red-75': '#fee2e2',
        'red-100': '#fecaca',
        'red-200': '#fca5a5',
        'red-400': '#f87171',
        'red-600': '#ef4444',
        'red-800': '#dc2626',
        'red-1000': '#b91c1c',

        'gray-50': '#fafafb',
        'gray-75': '#f5f5f7',
        'gray-100': '#f4f4f5',
        'gray-200': '#e5e6e9',
        'gray-400': '#b9bac1',
        'gray-600': '#7f8083',
        'gray-800': '#4b4c4f',
        'gray-1000': '#1a1b1d',

        'blue-50': '#e4f1ff',
        'blue-75': '#d5e7ff',
        'blue-100': '#cad3fd',
        'blue-200': '#b2bdfd',
        'blue-400': '#3354ff',
        'blue-600': '#2d3f9a',
        'blue-800': '#1d2963',
        'blue-1000': '#151d4b',
      },
      minWidth: {
        'screen-md': '768px',
      },
      flex: {
        1: '1 1 0%',
        2: '2 2 0%',
        3: '3 3 0%',
        4: '4 4 0%',
        5: '5 5 0%',
        6: '6 6 0%',
        7: '7 7 0%',
        8: '8 8 0%',
        9: '9 9 0%',
        10: '10 10 0%',
      },
      spacing: {
        overlay: 'overlay',
      },
    },
  },
  plugins: [require('@tailwindcss/typography'), require('daisyui')],
  daisyui: {
    themes: ['winter'],
  },
};
