// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
'use strict';
import * as vscode from 'vscode';
import { HelloWorldPanel } from './HelloWorldPanel';
import * as Net from 'net';
import { activateMockDebug } from './activateMockDebug';

const runMode: 'external' | 'server' | 'namedPipeServer' | 'inline' = 'server';

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
	console.log('Congratulations, your extension "testinghi" is now active!');

	context.subscriptions.push(
		vscode.commands.registerCommand("extension.vsname.helloWorld", () => {
			HelloWorldPanel.createOrShow(context.extensionUri);
		})
	);

	// debug adapters can be run in different ways by using a vscode.DebugAdapterDescriptorFactory:
	switch (runMode) {
		case 'server':
			// run the debug adapter as a server inside the extension and communicate via a socket
			activateMockDebug(context, new MockDebugAdapterServerDescriptorFactory());
			break;

		case 'external': default:
			// run the debug adapter as a separate process
			//activateMockDebug(context, new DebugAdapterExecutableFactory()); #
			break;

		case 'inline':
			// run the debug adapter inside the extension and directly talk to it
			activateMockDebug(context);
			break;
	}

}

// This method is called when your extension is deactivated
export function deactivate() {}

class MockDebugAdapterServerDescriptorFactory implements vscode.DebugAdapterDescriptorFactory {

	private server?: Net.Server;

	createDebugAdapterDescriptor(session: vscode.DebugSession, executable: vscode.DebugAdapterExecutable | undefined): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {

		// make VS Code connect to debug server
		return new vscode.DebugAdapterServer(63321);
	}

	dispose() {
		if (this.server) {
			this.server.close();
		}
	}
}