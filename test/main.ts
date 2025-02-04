import { BrowserWindow, app, dialog, ipcMain, nativeTheme } from "electron";
import os from "os";
import path from "path";
import fs from "fs";
import { fs as fs2, clipboard, Progress, shell, drag } from "../lib/index";

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

    shell.register(getHandle());
    // const vols = fs2.listVolumes();
    // console.log(vols);

    // let s = new Date().getTime();
    // const x = fs.readdirSync(__dirname, { withFileTypes: true, recursive: true });
    // x.forEach((a) => {
    //     const y = fs2.getFileAttribute(path.join(a.parentPath, a.name));
    //     if (y.isSystem || y.isHidden || y.isSymbolicLink) {
    //         console.log(a.name);
    //     }
    // });
    // console.log(x.length);
    // console.log(new Date().getTime() - s);

    // s = new Date().getTime();
    // const entries = fs2.readdir(__dirname, true, true);

    // console.log(entries.length);
    // console.log(new Date().getTime() - s);

    // const stat = fs.statSync(entries[0].fullPath);
    // console.log(stat);
    // const a = fs2.getFileAttribute(entries[0].fullPath);
    // console.log(a);
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

const getHandle = () => {
    const hwndBuffer = win.getNativeWindowHandle();
    let hwnd = 0;
    if (os.endianness() == "LE") {
        hwnd = hwndBuffer.readInt32LE();
    } else {
        hwnd = hwndBuffer.readInt32BE();
    }
    return hwnd;
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

let write = false;
const append = () => {
    write = !write;
    const hwnd = getHandle();
    if (write) {
        const text = "People’s textあ";
        clipboard.writeText(hwnd, text);
    } else {
        const result = clipboard.readText(hwnd);
        console.log(`result:${result}`);
        fs.writeFileSync(path.join(__dirname, "test.txt"), result, { encoding: "utf-8" });
    }
};

const toggle = () => {
    const hwnd = getHandle();
    write = !write;
    if (write) {
        clipboard.writeUris(hwnd, [path.join(__dirname, "..", "package.json"), path.join(__dirname, "..", "tsconfig.json")], "Copy");
    } else {
        let x = clipboard.readUris(hwnd);
        console.log(x);
    }
};

const reload = async (_e: any, _s: string[], _d: string) => {
    // try {
    //     sync ? await fs2.mvAll(s, d, progressCb) : await fs2.mvAll(s, d);
    // } catch (ex: any) {
    //     dialog.showErrorBox("e", ex.message);
    // }
    shell.getOpenWith(path.join(__dirname, "package.json"));

    // shell.openPathWith(path.join(__dirname, "package.json"), x[0].path);
};

const openprop = false;
const open = () => {
    if (openprop) {
        shell.openFileProperty(path.join(__dirname, "..", "package.json"));
    } else {
        shell.openPath(path.join(__dirname, "..", "package.json"));
    }
};

const openWith = false;
const openwith = () => {
    if (openWith) {
        shell.showOpenWithDialog(path.join(__dirname, "..", "package.json"));
    } else {
        shell.showItemInFolder(path.join(__dirname, "..", "package.json"));
    }
};

const content = () => {
    ["a.mp4", "a.json", "a.js", "a.ts", "a.mov"].forEach((e) => {
        const type = fs2.getMimeType(e);
        console.log(type);
    });
};

const draggable = () => {
    const f = __dirname;
    const a = [path.join(f, "main.ts"), path.join(f, "package.json"), path.join(f, "preload.js")];
    const hwnd = getHandle();
    drag.startDrag(a, hwnd);
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
    ipcMain.on("draggable", draggable);
});

app.on("window-all-closed", () => {
    if (process.platform !== "darwin") app.quit();
});
