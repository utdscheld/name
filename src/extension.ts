// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import * as dap from '@vscode/debugadapter';
import { HelloWorldPanel } from './HelloWorldPanel';
import { spawn } from "child_process";
import path = require('path');

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
	console.log('Congratulations, your extension "testinghi" is now active!');

	const text = `
		Content-Length: 206

		{
		  "seq": 152,
		  "type": "request",
		  "command": "initialize",
		  "arguments": {
		    "adapterID": "0001e357-72c7-4f03-ae8f-c5b54bd8dabf",
		    "clientName": "Some Cool Editor"
		  }
		}
		`;

	//process.stdout.write(text);

	//console.log(new dap.InitializedEvent());
//
	//const child = spawn("cmd.exe", ["/c", "cd c:\\Users\\wells\\OneDrive\\Documents\\GitHub\\name\\name-emu & cargo run"], {
	//	cwd: process.cwd(),
	//});
//
	//console.log(new dap.InitializedEvent());
	//console.log("I am daping");
//
	////child.stdin.write(new dap.InitializedEvent());
//
	//child.stdout.on("data", (data) => {
	//	console.log(data.toString());
	//});
//
	//child.stderr.on("data", (data) => {
	//	console.error(data.toString());
	//});
	//  
	//child.on("exit", (code) => {
	//	console.log(`Child process exited with code ${code}`);
	//});
	//console.log(new dap.InitializedEvent());
//
	//var runCmd = path.join(__dirname, "..\\name-emu");
	//runCmd = runCmd.concat(" & cargo run")
	//console.log(runCmd);
//
	//const child = spawn("cmd.exe", ["/c", runCmd.toString()], {
	//	cwd: process.cwd(),
	//});
//
	//console.log(new dap.InitializedEvent());
	//console.log("I am daping");
//
	////child.stdin.write(new dap.InitializedEvent());
//
	//child.stdout.on("data", (data) => {
	//	console.log(data.toString());
	//});
//
	//child.stderr.on("data", (data) => {
	//	console.error(data.toString());
	//});
	//  
	//child.on("exit", (code) => {
	//	console.log(`Child process exited with code ${code}`);
	//});

	//exec("cd c:\\Users\\wells\\OneDrive\\Documents\\GitHub\\name\\name-emu & cargo run", (err, stdout, stderr) => {
	//	if (err) {
	//	  console.error(err);
	//	  console.log("I got here ig.");
	//	  return;
	//	}
	//  
	//	console.log("I got here!");
	//	console.log(stdout);
	//});
	console.log("I got here too!");

	context.subscriptions.push(
		vscode.commands.registerCommand("vsname2.helloWorld", () => {
			HelloWorldPanel.createOrShow(context.extensionUri);
		})
	);
}

// This method is called when your extension is deactivated
export function deactivate() {}