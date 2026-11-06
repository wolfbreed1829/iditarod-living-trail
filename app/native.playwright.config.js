const {defineConfig}=require('@playwright/test');
module.exports=defineConfig({testDir:'./tests',testMatch:'native-cms.spec.js',timeout:60000,workers:1,
 use:{baseURL:'http://localhost:3110',channel:'chrome',headless:true,viewport:{width:1440,height:1000},trace:'retain-on-failure',screenshot:'only-on-failure'},
 reporter:[['list'],['json',{outputFile:'artifacts/native-tests/results.json'}]],outputDir:'artifacts/native-tests/output'});
