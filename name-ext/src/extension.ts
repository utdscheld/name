// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
'use strict';
import * as vscode from 'vscode';
import { HelloWorldPanel } from './HelloWorldPanel';
import * as Net from 'net';
import { activateNameDebug } from './activateNameDebug';

const runMode: 'external' | 'server' | 'namedPipeServer' | 'inline' = 'server';

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
	console.log('Congratulations, your extension "testinghi" is now active!');

	context.subscriptions.push(
		vscode.commands.registerCommand("extension.vsname.helloWorld", () => {
			HelloWorldPanel.createOrShow(context.extensionUri);
		}),
		vscode.commands.registerCommand('extension.vsname.openTerminal', () => {
			const terminal = vscode.window.createTerminal('NAME Emulator');
			terminal.show();
			terminal.sendText('echo Hello, Terminal!');
			terminal.sendText('cd C:\\Users\\wells\\OneDrive\\Documents\\GitHub\\mainName\\name\\name-emu');
			terminal.sendText('cargo build --release');
			terminal.sendText('cargo run 63321');

			// Listen for the "Port is ready" message in the terminal
			// const terminalNameToCheck = 'NAME Emulator';
    		// const disposable = vscode.window.onDidChangeActiveTerminal(activatedTerminal => {
       		// if (activatedTerminal && activatedTerminal.name === terminalNameToCheck) {
       		//     const exitListener = vscode.window.onDidCloseTerminal(terminal => {
       		//         if (terminal === activatedTerminal) {
       		//             // Execute the second command when the terminal is closed
       		//             vscode.commands.executeCommand('extension.vsname.connectToEmulator');
       		//             disposable.dispose();
       		//             exitListener.dispose();
       		//         }
            // });
// 
            // activatedTerminal.sendText('echo "Port is ready"'); // Send a check message
       		// }
    		// });
		}),
		vscode.commands.registerCommand('extension.vsname.commandTest', () => {
			const terminal = vscode.window.createTerminal('NAME Emulator - Manual');
			terminal.show();
			terminal.sendText('echo Hello, Terminal!');
			terminal.sendText('cd C:\\Users\\wells\\OneDrive\\Documents\\GitHub\\mainName\\name\\name-emu');
			terminal.sendText('cargo build --release');
			terminal.sendText('cargo run 63321');
		}),
		vscode.commands.registerCommand('extension.vsname.connectToEmulator', () => {
			console.log("connectToEmulator called");
			activateNameDebug(context, new NameDebugAdapterServerDescriptorFactory());
		})
	);
	

	// debug adapters can be run in different ways by using a vscode.DebugAdapterDescriptorFactory:
	switch (runMode) {
		case 'server':
			activateNameDebug(context, new NameDebugAdapterServerDescriptorFactory());
			break;

		case 'external': default:
			// run the debug adapter as a separate process
			//activateNameDebug(context, new DebugAdapterExecutableFactory());
			break;

		case 'inline':
			// run the debug adapter inside the extension and directly talk to it
			activateNameDebug(context);
			break;
	}

}

// This method is called when your extension is deactivated
export function deactivate() {}

class NameDebugAdapterServerDescriptorFactory implements vscode.DebugAdapterDescriptorFactory {

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