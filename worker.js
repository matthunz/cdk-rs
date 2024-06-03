const cdk = require('aws-cdk-lib');
const s3 = require('aws-cdk-lib/aws-s3');

const readline = require('readline');

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false
});

rl.on('line', (line) => {
    var app;

    try {
        const message = JSON.parse(line);
        const response = { json: eval(message.js) };
        console.log(JSON.stringify(response));
    } catch (error) {
        console.error('Failed to parse JSON:', error);
    }
});

