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
        'gray-1000': '#1A1B1D',
      },
      textColor: {
        'gray-50': '#FAFAFB',
        'gray-75': '#F5F5F7',
        'gray-200': '#E5E6E9',
        'gray-600': '#7F8083',
        'gray-1000': '#1A1B1D',
      },
      borderColor: {
        'gray-50': '#FAFAFB',
        'gray-75': '#F5F5F7',
        'gray-200': '#E5E6E9',
        'gray-600': '#7F8083',
        'gray-1000': '#1A1B1D',
      },
      minWidth: {
        'screen-md': '768px',
      },
    },
  },
  plugins: [require('@tailwindcss/typography'), require('daisyui')],
  daisyui: {
    themes: ['winter'],
  },
};
