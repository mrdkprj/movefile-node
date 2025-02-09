"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
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
Object.defineProperty(exports, "__esModule", { value: true });
exports.drag = exports.clipboard = exports.shell = exports.fs = void 0;
var MoveFile = __importStar(require("../build/index"));
var fs = /** @class */ (function () {
    function fs() {
    }
    var _a;
    _a = fs;
    fs.mv = function (from, to) { return __awaiter(void 0, void 0, void 0, function () {
        return __generator(_a, function (_b) {
            return [2 /*return*/, MoveFile.mv(from, to)];
        });
    }); };
    fs.mvAll = function (from, to) { return __awaiter(void 0, void 0, void 0, function () {
        return __generator(_a, function (_b) {
            return [2 /*return*/, MoveFile.mv_all(from, to)];
        });
    }); };
    fs.listVolumes = function () {
        return MoveFile.list_volumes();
    };
    fs.getFileAttribute = function (filePath) {
        return MoveFile.get_file_attribute(filePath);
    };
    fs.readdir = function (directory, recursive, withMimeType) {
        return MoveFile.readdir(directory, recursive, withMimeType);
    };
    fs.getMimeType = function (filePath) {
        return MoveFile.get_mime_type(filePath);
    };
    fs.copy = function (from, to) {
        return MoveFile.copy(from, to);
    };
    fs.copyAll = function (from, to) {
        return MoveFile.copy_all(from, to);
    };
    return fs;
}());
exports.fs = fs;
var shell = /** @class */ (function () {
    function shell() {
    }
    shell.trash = function (file) {
        return MoveFile.trash(file);
    };
    shell.openPath = function (filePath) {
        return MoveFile.open_path(filePath);
    };
    shell.openPathWith = function (filePath, appPath) {
        return MoveFile.open_path_with(filePath, appPath);
    };
    shell.openFileProperty = function (filePath) {
        return MoveFile.open_file_property(filePath);
    };
    shell.showItemInFolder = function (filePath) {
        return MoveFile.show_item_in_folder(filePath);
    };
    shell.getOpenWith = function (filePath) {
        return MoveFile.get_open_with(filePath);
    };
    shell.showOpenWithDialog = function (filePath) {
        return MoveFile.show_open_with_dialog(filePath);
    };
    shell.register = function (windowHandle) {
        MoveFile.register(windowHandle);
    };
    return shell;
}());
exports.shell = shell;
var clipboard = /** @class */ (function () {
    function clipboard() {
    }
    clipboard.isTextAvailable = function () {
        MoveFile.is_text_available();
    };
    clipboard.readText = function (windowHandle) {
        return MoveFile.read_text(windowHandle);
    };
    clipboard.writeText = function (windowHandle, text) {
        return MoveFile.write_text(windowHandle, text);
    };
    clipboard.isUrisAvailable = function () {
        MoveFile.is_uris_available();
    };
    clipboard.readUris = function (windowHandle) {
        return MoveFile.read_uris(windowHandle);
    };
    clipboard.writeUris = function (windowHandle, fullPaths, operation) {
        return MoveFile.write_uris(windowHandle, fullPaths, operation);
    };
    return clipboard;
}());
exports.clipboard = clipboard;
var drag = /** @class */ (function () {
    function drag() {
    }
    drag.startDrag = function (paths, windowHandle) {
        return MoveFile.start_drag(paths, windowHandle !== null && windowHandle !== void 0 ? windowHandle : 0);
    };
    return drag;
}());
exports.drag = drag;
