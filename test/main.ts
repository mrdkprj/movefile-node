import { BrowserWindow, app, ipcMain, nativeTheme } from "electron";

import path from "path";
import fs from "fs";
import { mv, cancel, mvSync, Progress } from "../lib/index";

let id = -1;
let sync = false;
let win: BrowserWindow;

const createWindow = () => {
    nativeTheme.themeSource = "dark";
    win = new BrowserWindow({
        title: "main",
        width: 800,
        height: 601,
        // darkTheme:true,

        webPreferences: {
            preload: path.join(__dirname, "preload.js"),
        },
    });

    win.loadFile("index.html");
};

const handleSetTitle = async (_e: any, s: string, d: string) => {
    count = 0;
    console.log("from");
    if (fs.existsSync(s)) {
        console.log(s);
        id = sync ? mvSync(s, d) : mv(s, d, progressCb);
        // id = sync ? mvSync(s, d) : mv(s, d);
    } else {
        console.log(d);
        id = sync ? mvSync(d, s) : mv(d, s, progressCb);
        // id = sync ? mvSync(d, s) : mv(d, s);
    }
    console.log(id);
    // cancel(id);
};
let count = 0;
const progressCb = (progress: Progress) => {
    count++;

    // if (count > 3) {
    //     cancel(id);
    // }
    const current = (progress.transferred / progress.totalFileSize) * 100;

    if (win) {
        win.webContents.send("progress", { current });
    }
};

const toggle = () => {
    if (id >= 0) {
        cancel(id);
    }
};

app.whenReady().then(async () => {
    createWindow();
    ipcMain.on("set-title", handleSetTitle);
    ipcMain.on("toggle", toggle);
    ipcMain.on("append", toggle);
    ipcMain.on("reload", toggle);
});

app.on("window-all-closed", () => {
    if (process.platform !== "darwin") app.quit();
});
