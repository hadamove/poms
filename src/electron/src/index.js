const { app, BrowserWindow } = require('electron');
const path = require('path');


app.commandLine.appendSwitch('enable-unsafe-webgpu')

const createWindow = () => {
    const mainWindow = new BrowserWindow({
        width: 800,
        height: 600,
    });

    mainWindow.loadFile(path.join(__dirname, '../../../index.html'));
    mainWindow.webContents.openDevTools();
};

app.on('ready', createWindow);

app.on('window-all-closed', () => {
    app.quit();
});
