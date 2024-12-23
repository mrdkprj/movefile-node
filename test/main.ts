import { BrowserWindow, app, dialog, ipcMain, nativeTheme } from "electron";
import os from "os";
import path from "path";
import fs from "fs";
import * as nostd from "../lib/index";

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

    const vols = nostd.listVolumes();
    console.log(vols);

    // const hwndBuffer = win.getNativeWindowHandle();
    // let hwnd = 0;
    // if (os.endianness() == "LE") {
    //     hwnd = hwndBuffer.readInt32LE();
    // } else {
    //     hwnd = hwndBuffer.readInt32BE();
    // }
    // const x = readUrlsFromClipboard(hwnd);
    // console.log(x);

    // const y = readText(hwnd);
    // console.log(y);
};

const handleSetTitle = async (_e: any, s: string, d: string) => {
    console.log("from");
    try {
        if (fs.existsSync(s)) {
            console.log(s);
            sync ? await nostd.mv(s, d, progressCb) : await nostd.mv(s, d);
        } else {
            console.log(d);
            sync ? await nostd.mv(d, s, progressCb) : await nostd.mv(d, s);
        }
    } catch (ex: any) {
        console.log("error");
        console.log(ex);

        dialog.showErrorBox("e", ex.message);
    }
    // cancel(id);
};
let count = 0;
const progressCb = (progress: nostd.Progress) => {
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
                const x = nostd.getFileAttribute(path.join(directory, dirent.name));

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
        sync ? await nostd.mvBulk(s, d, progressCb) : await nostd.mvBulk(s, d);
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
    nostd.writeUrlsToClipboard(hwnd, [path.join(__dirname, "..", "package.json"), path.join(__dirname, "..", "tsconfig.json")], "Move");
};

let openprop = false;
const open = () => {
    const hwndBuffer = win.getNativeWindowHandle();
    let hwnd = 0;
    if (os.endianness() == "LE") {
        hwnd = hwndBuffer.readInt32LE();
    } else {
        hwnd = hwndBuffer.readInt32BE();
    }
    console.log(path.join(__dirname, "..", "package.json"));
    if (openprop) {
        nostd.openFileProperty(hwnd, path.join(__dirname, "..", "package.json"));
    } else {
        nostd.openPath(hwnd, path.join(__dirname, "..", "package.json"));
    }
};

app.whenReady().then(async () => {
    createWindow();
    ipcMain.on("set-title", handleSetTitle);
    ipcMain.on("toggle", toggle);
    ipcMain.on("append", append);
    ipcMain.on("reload", reload);
    ipcMain.on("open", open);
});

app.on("window-all-closed", () => {
    if (process.platform !== "darwin") app.quit();
});
