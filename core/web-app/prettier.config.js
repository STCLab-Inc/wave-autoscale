// Prettier configuration for the web-app
module.exports = {
  plugins: [require('prettier-plugin-tailwindcss')],
  tailwindConfig: './tailwind.config.js',
  singleQuote: true,
  tabWidth: 2,
  semi: true,
};
