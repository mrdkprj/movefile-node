import { BrowserWindow, app, dialog, ipcMain, nativeTheme } from "electron";
import os from "os";
import path from "path";
import fs from "fs";
import { mv, Progress, mvBulk, readUrlsFromClipboard, getFileAttribute, writeUrlsToClipboard, listVolumes } from "../lib/index";

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

    const vols = listVolumes();
    console.log(vols);
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
    console.log("from");
    try {
        if (fs.existsSync(s)) {
            console.log(s);
            sync ? await mv(s, d, progressCb) : await mv(s, d);
        } else {
            console.log(d);
            sync ? await mv(d, s, progressCb) : await mv(d, s);
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

const append = () => {
    const directory = __dirname;

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
    try {
        sync ? await mvBulk(s, d, progressCb) : await mvBulk(s, d);
    } catch (ex: any) {
        dialog.showErrorBox("e", ex.message);
    }
};

const toggle = () => {
    const hwndBuffer = win.getNativeWindowHandle();
    let hwnd = 0;
    if (os.endianness() == "LE") {
        hwnd = hwndBuffer.readInt32LE();
    } else {
        hwnd = hwndBuffer.readInt32BE();
    }

    writeUrlsToClipboard(hwnd, ["C:\\DevProjects\\fs3.rs", "C:\\DevProjects\\fs2.rs"], "Move");
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
