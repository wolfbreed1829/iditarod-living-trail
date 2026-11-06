const { defineConfig } = require('@playwright/test');
module.exports = defineConfig({
  testDir: './tests', testMatch: 'demo.spec.js', timeout: 60000, workers: 2,
  use: { baseURL: process.env.DEMO_URL || 'http://localhost:3107', channel: 'chrome', headless: true, viewport: {width:1440,height:1000}, screenshot:'only-on-failure', trace:'retain-on-failure' },
  reporter: [['list'],['json',{outputFile:'test-results/report.json'}]],
});
