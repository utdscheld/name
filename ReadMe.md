# How to run - First Install
## First open name-as in a seperate window and run it
- This is in the same branch
- In "name-as" make a directory called ".artifacts"
    - Add your .asm file you are using for testing to this directory
- Run "make build"
- Run "make"

## Then open name-emu in a seperate window and run it
- This is in the same branch
- We need to manually change a hard coded path for now
    - Search for "reset_mips" in main.rs
    - Change the path in "let program_data" to point to the same file but on your machine
        - This is "output.o" in the name-as directory
- Run "cargo build --release" to build the emulator
- Open main.rs then press f5 to run the emulator

## Note: This extension listens for name-emu on port 63321
## Now to build and run this extension
- This is in the same branch
- Run "npm install" to install required node modules
- Run "npm run watch" to build the extension and watch for changes
- Open extension.ts and press f5 now to open the development window
- Say "debug anyway"
- Open the folder with your mips file you are using for testing
- Open the .asm file, then press f5 to start debugging
    - If you get an error saying "ERCONREFUSED" then an ip, you don't have the emulator running, run it
- Ignore errors, if u see regsiters you're good
    - There is lots of hard coded stuff at the moment