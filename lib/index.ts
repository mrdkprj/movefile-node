import * as MoveFile from "../build/index";

export type Progress = {
    totalFileSize: number;
    transferred: number;
};
export type ProgressCallback = (progress: Progress) => void;

export const mv = (sourceFile: string, destFile: string, callback?: ProgressCallback): number => {
    if (callback) {
        return MoveFile.mv(sourceFile, destFile, callback);
    } else {
        return MoveFile.mv(sourceFile, destFile);
    }
};

export const mvSync = (sourceFile: string, destFile: string): number => {
    return MoveFile.mvSync(sourceFile, destFile);
};

export const cancel = (id: number): boolean => {
    return MoveFile.cancel(id);
};
