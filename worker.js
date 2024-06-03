const cdk = require('./package');


const readline = require('readline');

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false
});

rl.on('line', (line) => {
    try {
        const message = JSON.parse(line);
        const response = { json: eval(message.js) };
        console.log(JSON.stringify(response));
    } catch (error) {
        console.error('Failed to parse JSON:', error);
    }
});

