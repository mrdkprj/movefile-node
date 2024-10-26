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
Object.defineProperty(exports, "__esModule", { value: true });
exports.cancel = exports.mvSync = exports.mv = void 0;
var MoveFile = __importStar(require("../build/index"));
var mv = function (sourceFile, destFile, callback) {
    if (callback) {
        return MoveFile.mv(sourceFile, destFile, callback);
    }
    else {
        return MoveFile.mv(sourceFile, destFile);
    }
};
exports.mv = mv;
var mvSync = function (sourceFile, destFile) {
    return MoveFile.mvSync(sourceFile, destFile);
};
exports.mvSync = mvSync;
var cancel = function (id) {
    return MoveFile.cancel(id);
};
exports.cancel = cancel;
