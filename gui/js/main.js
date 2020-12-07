import addon from '../native/index.node';

import showDialog from 'rust-greenhorn/js/electron_dialogs';

const {app, BrowserWindow, Menu, ipcMain} = require('electron');
process.env.ELECTRON_DISABLE_SECURITY_WARNINGS = true;


function createWindow () {
  const port = addon.run();
  global.port = port;
  let width = 1100;
  if (process.env.NODE_ENV === "development") {
    width = 1900;
  }

  Menu.setApplicationMenu(null);
  const mainWindow = new BrowserWindow({
    width: width,
    height: 1000,
    webPreferences: {
      webSecurity: false,
      nodeIntegration: true
    }
  });
  mainWindow.loadURL(`file://${__dirname}/index.html`);
  if (process.env.NODE_ENV === "development") {
    mainWindow.webContents.openDevTools();
  }
}

app.whenReady().then(() => {
  createWindow();
  app.on('activate', function () {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
})

app.on('window-all-closed', function () {
  if (process.platform !== 'darwin') {
      app.quit();
  }
})

ipcMain.on('dialog', (event, arg) => {
  showDialog(arg, (result) => {
    event.reply('dialog', result);
  });
});