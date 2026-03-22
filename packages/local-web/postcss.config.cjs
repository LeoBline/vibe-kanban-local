const path = require('path');

module.exports = {
  plugins: {
    tailwindcss: { config: path.join(__dirname, 'tailwind.new.config.js') },
    autoprefixer: {},
  },
};
