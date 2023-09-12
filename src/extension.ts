// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import { HelloWorldPanel } from './HelloWorldPanel';

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
	console.log('Congratulations, your extension "testinghi" is now active!');

	context.subscriptions.push(
		vscode.commands.registerCommand("vsname2.helloWorld", () => {
			HelloWorldPanel.createOrShow(context.extensionUri);
		})
	);
}

// This method is called when your extension is deactivated
export function deactivate() {}
