import * as MoveFile from "../build/index";

export type Progress = {
    totalFileSize: number;
    transferred: number;
};
export type ProgressCallback = (progress: Progress) => void;

export const mv = (sourceFile: string, destFile: string, callback?: ProgressCallback): number => {
    return MoveFile.mv(sourceFile, destFile, callback);
};

export const mvSync = (sourceFile: string, destFile: string): number => {
    return MoveFile.mvSync(sourceFile, destFile);
};

export const cancel = (id: number): boolean => {
    return MoveFile.cancel(id);
};
