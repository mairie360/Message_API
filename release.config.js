// release.config.js à la racine de Message_API
module.exports = {
  branches: ['main'],
  plugins: [
    '@semantic-release/commit-analyzer',
    '@semantic-release/release-notes-generator',
    ['semantic-release-cargo', { 'publish': false }], // On ne publie pas sur crates.io
    '@semantic-release/github'
  ]
};
