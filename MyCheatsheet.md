# Setting up NPM
Ensure you have the latest version of NPM
- Install NVM
	- `curl -o- https://raw.githubusercontent.com/nvm sh/nvm/v0.39.1/install.sh | bash ``
- Install NodeJS
	- `nvm install --lts`
	- `nvm use --lts`
	- `npm install -g npm@latest`
- Install Yeoman, this is used to set up the basic extension
	- `npm install -g yo`
# Now to creating the extension
Run `yo code`
- Go through the set up
	- Should all be defaults, name is whatever
	- Enabling webpack might make publishing it easier but will also make you have to remove the tasks below

**Open the extension's code in VSCode**
- Remove the tasks from tasks.json
	```json
	...
		// See https://go.microsoft.com/fwlink/?LinkId=733558
		// for the documentation about the tasks.json format
		{
		"version": "2.0.0",
		"tasks": [
		]
		}
	...
	```
	It should look like this
- Remove the default build task from launch.json
	```json
	...
	configurations": [
	{
	"name": "Run Extension",
	"type": "extensionHost",
	"request": "launch",
	"args": [
	"--extensionDevelopmentPath=${workspaceFolder}"
	],
	"outFiles": [
	"${workspaceFolder}/dist/**/*.js"
	],
	},
	{
	"name": "Extension Tests",
...
	```
	It should look like this
# RUN THIS EVERY TIME YOU START CODING
```sh
npm run watch
```
Keep this running in a terminal window while developing

- If you get a permission denied error when running `npm run watch`, do the following
	- Delete /node_modules
	- Delete package-lock.json
	- Run `npm install`
	- Now run `npm run watch` again
# Extension Window
This is where you can see how the extension works

Press `F5` to launch the extension window

## Specified Task Cannot be Tracked
This may appear the first time you open the window

If it does, click "Configure Task"

## Continued

Every time you make changes to the extension, you need to reload the extension window
- Do this by opening the command pallet (`ctrl + shift + p`)  and then entering `Developer: Reload Window`
	- You can also just hit `ctrl + r`

After reloading the window, you can enter any **commands** you have added into the command pallet


# Creating a new command
Start by adding a command in the `extensions.ts` file
```ts
context.subscriptions.push(
	vscode.commands.registerCommand('vs-name.helloWorld', () => {
		vscode.window.showInformationMessage('Hello from vs-name!');
	})
);
```
- It goes in the `export function activate...` block

Change the premade command to be that structure

**How the question works**
```ts
vscode.window.showInformationMessage("How was your day?", "good", "bad");
```
This lets us pop up a dialog box to a user and ask a question
- First string is question
- Next 2 strings are answers the user can choose from

This is 

**Adding command to package.json**
We need to do this after creating it in extensions.ts
```json
"commands": [
      {
        "command": "vsname.helloWorld",
        "title": "Hello World"
      },
      {
        "command": "vsname.askQuestion",
        "category": "VSNAME",
        "title": "Ask Question"
      }
    ]
```
The second command is the command we created above

**VSCode creates activation events automatically**
- You don't have to add these

## Editing the command to do something with our question answers
```ts
context.subscriptions.push(
        vscode.commands.registerCommand("vsname.askQuestion", async () => {
            const answer = await vscode.window.showInformationMessage(
                "How was your day?",
                "good",
                "bad"
            );
            if (answer === 'bad') {
                vscode.window.showInformationMessage("Sucks for you then");
            } else {
                console.log({ answer });
            }
        })
    );
```

# Now adding a panel
We copied the [swiper panel code here](https://github.com/benawad/vsinder/blob/master/packages/extension/src/SwiperPanel.ts)
We also copy the [reset.css](https://github.com/microsoft/vscode-extension-samples/blob/main/webview-sample/media/reset.css) and the [vscode.css](https://github.com/microsoft/vscode-extension-samples/blob/main/webview-sample/media/vscode.css) from the [vscode-extension-samples](https://github.com/microsoft/vscode-extension-samples/tree/main) [cat example](https://github.com/microsoft/vscode-extension-samples/tree/main/webview-sample). We are also getting the `getNonce()` function from the [extensions.ts](https://github.com/microsoft/vscode-extension-samples/blob/main/webview-sample/src/extension.ts) of this example.
- We will put `reset.css` and `vscode.css` in a new folder called `media`
- This makes the panel look like a native vscode panel
- `getNonce()` will go in a new file called `getNonce.ts`
	- That will contain a single export with the `getNonce()` function
	- `getNonce()` returns a unique id
- Make sure to import `getNonce()`, then remove the other imports for now

HTML code goes at the bottom of `HelloWorldPanel.ts`

-----

# Now for Javascript!
We used the `main.js` from the [cat example](https://github.com/microsoft/vscode-extension-samples/tree/main/webview-sample) 
``` js
// This script will be run within the webview itself
// It cannot access the main VS Code APIs directly.

(function () {
	const vscode = acquireVsCodeApi();

	console.log("hello there");
}());
```

**How can we debug the js part?**
- `ctrl + shift + p`
- `openweb` then open webview developer tools should autopopulate
- This will show you js errors

-----

# Now for Svelte
Svelte is a JS framework that is easier to code in than pure JS, and compiles into vanilla JS so it is easy to run

**Svelte Setup**
We took the `rollup.config.js` file from the [vsinder repo](https://github.com/benawad/vsinder/blob/master/packages/extension/rollup.config.js) and put it in our main extension directory
- Change line 13 to say "webviews", "pages" instead of "svelte-stuff", "pages"
	- Change all references of "svelte-stuff" to "webviews" as well
- Create a webviews directory and under that a pages directory, put these in the main directory
- Install all the packages at the top of this file with npm
	- `npm i -D rollup-plugin-svelte @rollup/plugin-node-resolve @rollup/plugin-commonjs rollup-plugin-terser svelte-preprocess @rollup/plugin-typescript`
- Add `tsconfog.json` from the [vsinder repo](https://github.com/benawad/vsinder/blob/master/packages/extension/rollup.config.js) to the `webviews` folder
- Add some more Svelte packages
	- `npm i -D @tsconfig/svelte svelte svelte-check svelte-preprocess concurrently`