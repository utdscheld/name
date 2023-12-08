// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
'use strict';
import * as vscode from 'vscode';
import * as Net from 'net';
import { activateNameDebug } from './activateNameDebug';
import * as path from 'path';
const { spawn } = require('child_process');

const termName = "NAME Emulator";

const runMode: 'external' | 'server' | 'namedPipeServer' | 'inline' = 'server';

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
	context.subscriptions.push(
		vscode.commands.registerCommand("extension.vsname.startEmu", () => {
			// User configuration
			var configuration = vscode.workspace.getConfiguration('name-ext');
			if (!configuration) {
				vscode.window.showErrorMessage("Failed to find NAME configurations");
				return;
			}

			const namePath = configuration.get('namePath', '');
			if (namePath.length < 1) {
				vscode.window.showErrorMessage(`Failed to find a path for NAME, please set the path in VSCode's User Settings under name-ext`);
				return;
			}

			const nameASPath = path.join(namePath, 'name-as');
			const nameTMPPath = path.join(namePath, 'tmp');
			const nameDefaultCfgPath = path.join(nameASPath, 'configs/default.toml');
			const nameEMUPath = path.join(namePath, 'name-emu');
			const nameEXTPath = path.join(namePath, 'name-ext');

			// Start the extension with 'npm run watch'
			// We def don't need the watch feature in the prod distribution but we can remove that later
			const child = spawn(
				'npm', ['run', 'watch'], {
					cwd: nameEXTPath
				}
			);

			child.on('error', (_) => {
				vscode.window.showErrorMessage(`Failed to start name-ext, please ensure you have npm installed`);
			});

			child.on('exit', (code, _) => {
				if (code !== 0) {
					vscode.window.showErrorMessage(`name-ext exited with code ${code}`);
				}
			});

			var editor = vscode.window.activeTextEditor;			
			if (editor) {
				// Get currently-open file path
				var currentlyOpenTabFilePath = editor.document.fileName;
				var currentlyOpenTabFileName = path.basename(currentlyOpenTabFilePath);

				const terminalOptions = { name: termName, closeOnExit: true };
				var terminal = vscode.window.terminals.find(terminal => terminal.name === termName);
				terminal = terminal ? terminal : vscode.window.createTerminal(terminalOptions);
				terminal.show();
				terminal.sendText('clear');

				// Make the temp directory
				terminal.sendText(`cd ${namePath}`);
				terminal.sendText(`mkdir tmp`);

				// Build and run assembler
				terminal.sendText(`cd ${nameASPath}`);
				terminal.sendText('cargo build --release');
				terminal.sendText(`cargo run ${nameDefaultCfgPath} ${currentlyOpenTabFilePath} ${nameTMPPath}/${currentlyOpenTabFileName}.o -l`);
				
				// Build and run emulator
				terminal.sendText(`cd ${nameEMUPath}`);
				terminal.sendText('cargo build --release');
				terminal.sendText(`cargo run 63321 ${currentlyOpenTabFilePath} ${nameTMPPath}/${currentlyOpenTabFileName}.o ${nameTMPPath}/${currentlyOpenTabFileName}.o.li`);

				// Exit when emulator quits
				terminal.sendText('exit');
				terminal.sendText('cd ${namePath}');
				terminal.sendText('rm -r tmp');
			}

			// Kill child process if it's still alive
			if (child) {
				child.kill();
			}
		})
	);

	// debug adapters can be run in different ways by using a vscode.DebugAdapterDescriptorFactory:
	switch (runMode) {
		case 'server':
			// run the debug adapter as a server inside the extension and communicate via a socket
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
