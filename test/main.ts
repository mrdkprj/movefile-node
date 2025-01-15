import { BrowserWindow, app, dialog, ipcMain, nativeTheme } from "electron";
import os from "os";
import path from "path";
import fs from "fs";
import { fs as fs2, clipboard, Progress, shell } from "../lib/index";

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

    // const vols = fs2.listVolumes();
    // console.log(vols);

    let s = new Date().getTime();
    const x = fs.readdirSync(__dirname, { withFileTypes: true, recursive: true });
    // x.forEach((a) => {
    //     const y = fs2.getFileAttribute(path.join(a.parentPath, a.name));
    //     if (y.isSystem || y.isHidden || y.isSymbolicLink) {
    //         console.log(a.name);
    //     }
    // });
    console.log(x.length);
    console.log(new Date().getTime() - s);

    s = new Date().getTime();
    const entries = fs2.readdir(__dirname, true, true);

    console.log(entries.length);
    console.log(new Date().getTime() - s);
    // entries.forEach((entry) => console.log(entry.fullPath));

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
            sync ? await fs2.mv(s, d, progressCb) : await fs2.mv(s, d);
        } else {
            console.log(d);
            sync ? await fs2.mv(d, s, progressCb) : await fs2.mv(d, s);
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
                const x = fs2.getFileAttribute(path.join(directory, dirent.name));

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
                return !x.isSystem;
            } catch (ex: any) {
                console.log(path.join(directory, dirent.name));
                return true;
            }
        })
        .forEach((file) => files.push(file));
};

const reload = async (_e: any, s: string[], d: string) => {
    try {
        sync ? await fs2.mvAll(s, d, progressCb) : await fs2.mvAll(s, d);
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
    clipboard.writeUris(hwnd, [path.join(__dirname, "..", "package.json"), path.join(__dirname, "..", "tsconfig.json")], "Move");
};

const openprop = false;
const open = () => {
    const hwndBuffer = win.getNativeWindowHandle();
    let hwnd = 0;
    if (os.endianness() == "LE") {
        hwnd = hwndBuffer.readInt32LE();
    } else {
        hwnd = hwndBuffer.readInt32BE();
    }

    if (openprop) {
        shell.openFileProperty(hwnd, path.join(__dirname, "..", "package.json"));
    } else {
        shell.openPath(hwnd, path.join(__dirname, "..", "package.json"));
    }
};

const openWith = false;
const openwith = () => {
    const hwndBuffer = win.getNativeWindowHandle();
    let hwnd = 0;
    if (os.endianness() == "LE") {
        hwnd = hwndBuffer.readInt32LE();
    } else {
        hwnd = hwndBuffer.readInt32BE();
    }

    if (openWith) {
        shell.openPathWith(hwnd, path.join(__dirname, "..", "package.json"));
    } else {
        shell.showItemInFolder(path.join(__dirname, "..", "package.json"));
    }
};

const content = () => {
    const x = path.join(__dirname, "..", "package.json");
    const type = fs2.getMimeType(x);
    console.log(type);
};

app.whenReady().then(async () => {
    createWindow();
    ipcMain.on("set-title", handleSetTitle);
    ipcMain.on("toggle", toggle);
    ipcMain.on("append", append);
    ipcMain.on("reload", reload);
    ipcMain.on("open", open);
    ipcMain.on("openwith", openwith);
    ipcMain.on("content", content);
});

app.on("window-all-closed", () => {
    if (process.platform !== "darwin") app.quit();
});
