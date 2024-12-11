import { BrowserWindow, app, dialog, ipcMain, nativeTheme } from "electron";
import os from "os";
import path from "path";
import fs from "fs";
import { mv, cancel, mvSync, Progress, mvBulk, reserveCancellable, readUrlsFromClipboard, getFileAttribute } from "../lib/index";

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

    const hwndBuffer = win.getNativeWindowHandle();
    let hwnd = 0;
    if (os.endianness() == "LE") {
        hwnd = hwndBuffer.readInt32LE();
    } else {
        hwnd = hwndBuffer.readInt32BE();
    }
    const x = readUrlsFromClipboard(hwnd);
    console.log(x);
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

const append = () => {
    const directory = "D:\\";

    const allDirents = fs.readdirSync(directory, { withFileTypes: true });
    const files = [];
    allDirents
        .filter((dirent, i) => {
            try {
                const x = getFileAttribute(path.join(directory, dirent.name));
                if (i == 2) {
                    const s = fs.statSync(path.join(directory, dirent.name));
                    console.log(path.join(directory, dirent.name));
                    console.log(s.atimeMs);
                    console.log(x.atime);
                    console.log(s.mtimeMs);
                    console.log(x.mtime);
                    console.log(s.size);
                    console.log(x.size);
                }
                return !x.system;
            } catch (ex: any) {
                console.log(path.join(directory, dirent.name));
                return true;
            }
        })
        .forEach((file) => files.push(file));
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
