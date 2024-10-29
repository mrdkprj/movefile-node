import * as MoveFile from "../build/index";

export type Progress = {
    totalFileSize: number;
    transferred: number;
};
export type ProgressCallback = (progress: Progress) => void;

export const mv = async (sourceFile: string, destFile: string, callback?: ProgressCallback, id?: number) => {
    if (callback) {
        return await MoveFile.mv(sourceFile, destFile, callback, id);
    } else {
        return await MoveFile.mv(sourceFile, destFile, id);
    }
};

export const mvSync = (sourceFile: string, destFile: string): number => {
    return MoveFile.mvSync(sourceFile, destFile);
};

export const cancel = (id: number): boolean => {
    return MoveFile.cancel(id);
};

export const reserveId = (): number => {
    return MoveFile.reserve();
};
