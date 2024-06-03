const cdk = require("aws-cdk-lib");
const s3 = require("aws-cdk-lib/aws-s3");
const ec2 = require("aws-cdk-lib/aws-ec2");

const readline = require("readline");

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false,
});

var app;
var stacks = {};

rl.on("line", (line) => {
  try {
    const message = JSON.parse(line);
    try {
      const response = { json: eval(message.js) };
      console.log(JSON.stringify(response));
    } catch (error) {
      console.error("Failed to execute JS in worker:", error);
    }
  } catch (error) {
    console.error("Failed to parse JSON in worker:", error);
  }
});
