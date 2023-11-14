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
        'gray-50': '#FAFAFB',
        'gray-75': '#F5F5F7',
        'gray-200': '#E5E6E9',
        'gray-600': '#7F8083',
        'gray-800': '#4B4C4F',
        'gray-1000': '#1A1B1D',
      },
      textColor: {
        'gray-50': '#FAFAFB',
        'gray-75': '#F5F5F7',
        'gray-200': '#E5E6E9',
        'gray-600': '#7F8083',
        'gray-800': '#4B4C4F',
        'gray-1000': '#1A1B1D',
      },
      borderColor: {
        'gray-50': '#FAFAFB',
        'gray-75': '#F5F5F7',
        'gray-200': '#E5E6E9',
        'gray-600': '#7F8083',
        'gray-800': '#4B4C4F',
        'gray-1000': '#1A1B1D',
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
    },
  },
  plugins: [require('@tailwindcss/typography'), require('daisyui')],
  daisyui: {
    themes: ['winter'],
  },
};
