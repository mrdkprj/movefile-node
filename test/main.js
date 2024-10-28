"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
var electron_1 = require("electron");
var path_1 = __importDefault(require("path"));
var fs_1 = __importDefault(require("fs"));
var index_1 = require("../lib/index");
var id = -1;
var sync = false;
var win;
var createWindow = function () {
    electron_1.nativeTheme.themeSource = "dark";
    win = new electron_1.BrowserWindow({
        title: "main",
        width: 800,
        height: 601,
        // darkTheme:true,
        webPreferences: {
            preload: path_1.default.join(__dirname, "preload.js"),
        },
    });
    win.loadFile("index.html");
};
var handleSetTitle = function (_e, s, d) { return __awaiter(void 0, void 0, void 0, function () {
    var _a, _b, ex_1;
    return __generator(this, function (_c) {
        switch (_c.label) {
            case 0:
                count = 0;
                console.log("from");
                _c.label = 1;
            case 1:
                _c.trys.push([1, 10, , 11]);
                if (!fs_1.default.existsSync(s)) return [3 /*break*/, 5];
                console.log(s);
                if (!sync) return [3 /*break*/, 2];
                _a = (0, index_1.mvSync)(s, d);
                return [3 /*break*/, 4];
            case 2: return [4 /*yield*/, (0, index_1.mv)(s, d, progressCb)];
            case 3:
                _a = _c.sent();
                _c.label = 4;
            case 4:
                _a;
                return [3 /*break*/, 9];
            case 5:
                console.log(d);
                if (!sync) return [3 /*break*/, 6];
                _b = (0, index_1.mvSync)(d, s);
                return [3 /*break*/, 8];
            case 6: return [4 /*yield*/, (0, index_1.mv)(d, s, progressCb)];
            case 7:
                _b = _c.sent();
                _c.label = 8;
            case 8:
                _b;
                _c.label = 9;
            case 9:
                console.log(id);
                return [3 /*break*/, 11];
            case 10:
                ex_1 = _c.sent();
                console.log("error");
                console.log(ex_1);
                electron_1.dialog.showErrorBox("e", ex_1.message);
                return [3 /*break*/, 11];
            case 11: return [2 /*return*/];
        }
    });
}); };
var count = 0;
var progressCb = function (progress) {
    count++;
    console.log(progress);
    // if (count > 3) {
    //     cancel(id);
    // }
    var current = (progress.transferred / progress.totalFileSize) * 100;
    if (win) {
        win.webContents.send("progress", { current: current });
    }
};
var toggle = function () {
    if (id >= 0) {
        (0, index_1.cancel)(id);
    }
};
electron_1.app.whenReady().then(function () { return __awaiter(void 0, void 0, void 0, function () {
    return __generator(this, function (_a) {
        createWindow();
        electron_1.ipcMain.on("set-title", handleSetTitle);
        electron_1.ipcMain.on("toggle", toggle);
        electron_1.ipcMain.on("append", toggle);
        electron_1.ipcMain.on("reload", toggle);
        return [2 /*return*/];
    });
}); });
electron_1.app.on("window-all-closed", function () {
    if (process.platform !== "darwin")
        electron_1.app.quit();
});
