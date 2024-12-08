import { BrowserWindow, app, dialog, ipcMain, nativeTheme } from "electron";

import path from "path";
import fs from "fs";
import { mv, cancel, mvSync, Progress, trash, mvBulk, reserveCancellable, listVolumes, getFileAttribute } from "../lib/index";

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

    console.log(new Date());
    const x = listVolumes();
    console.log(x);
    const f = getFileAttribute("path");
    console.log(f);
    console.log(new Date());
    win.loadFile("index.html");
};

const handleSetTitle = async (_e: any, s: string, d: string) => {
    let id = reserveCancellable();
    console.log(id);
    const x = 10;
    if (x > 0) {
        return;
    }
    count = 0;
    console.log("from");
    try {
        if (fs.existsSync(s)) {
            console.log(s);
            sync ? mvSync(s, d) : await mv(s, d, progressCb);
            // sync ? mvSync(s, d) : await mv(s, d);
        } else {
            console.log(d);
            sync ? mvSync(d, s) : await mv(d, s, progressCb);
            // sync ? mvSync(d, s) : await mv(d, s);
        }
    } catch (ex: any) {
        console.log("error");
        console.log(ex);

        dialog.showErrorBox("e", ex.message);
    }
    // cancel(id);
};
let count = 0;
const progressCb = (progress: Progress) => {
    count++;

    // if (count > 3) {
    //     cancel(id);
    // }
    const current = (progress.transferred / progress.totalFileSize) * 100;
    console.log(progress);
    if (win) {
        win.webContents.send("progress", { current });
    }
};

const toggle = () => {
    if (id >= 0) {
        cancel(id);
    }
};

const append = (_e: any, s: string) => {
    trash(s);
};

const reload = async (_e: any, s: string[], d: string) => {
    console.log("reload");
    try {
        await mvBulk(s, d, progressCb);
    } catch (ex: any) {
        dialog.showErrorBox("e", ex.message);
    }
};

app.whenReady().then(async () => {
    createWindow();
    ipcMain.on("set-title", handleSetTitle);
    ipcMain.on("toggle", toggle);
    ipcMain.on("append", append);
    ipcMain.on("reload", reload);
});

app.on("window-all-closed", () => {
    if (process.platform !== "darwin") app.quit();
});
