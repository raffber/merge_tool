
var {remote, ipcRenderer} = require('electron');

import Application from 'rust-greenhorn/js/app.js';
import Pipe from 'rust-greenhorn/js/websocket.js';
import '../scss/main.scss';

let app = null;

window.onload = function() {
    const port = remote.getGlobal('port');
    let pipe = new Pipe("ws://127.0.0.1:" + port);
    app = new Application(pipe, document.body, (app, dialog) => {
        ipcRenderer.send('dialog', dialog);
    });
    window.app = app;
}

window.onunload = function() {
    app.pipe.close();
}

ipcRenderer.on('dialog', (event, arg) => {
    let result = {'Dialog': arg};
    window.app.sendReturnMessage(result);
});