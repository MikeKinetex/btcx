module.exports = {
  printWidth: 120,
  tabWidth: 2,
  useTabs: false,
  singleQuote: true,
  plugins: ["prettier-plugin-solidity"],
  overrides: [
    {
      files: '*.sol',
      options: {
        tabWidth: 4,
        singleQuote: false
      },
    },
  ],
};