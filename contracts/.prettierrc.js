module.exports = {
  printWidth: 160,
  tabWidth: 2,
  useTabs: false,
  singleQuote: true,
  overrides: [
    {
      files: '*.sol',
      options: {
        tabWidth: 4,
        singleQuote: false,
        bracketSpacing: false,
      },
    },
  ],
};