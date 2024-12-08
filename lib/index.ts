import * as MoveFile from "../build/index";

export type Progress = {
    totalFileSize: number;
    transferred: number;
};
export type ProgressCallback = (progress: Progress) => void;
export type Volume = {
    mountPoint: string;
    volumeLabel: string;
};
export type FileAttribute = {
    readOnly: boolean;
    hidden: boolean;
    system: boolean;
    device: boolean;
};

export const mv = async (sourceFile: string, destFile: string, callback?: ProgressCallback, id?: number) => {
    if (callback) {
        return await MoveFile.mv(sourceFile, destFile, callback, id);
    } else {
        return await MoveFile.mv(sourceFile, destFile, id);
    }
};

export const mvBulk = async (sourceFiles: string[], destDir: string, callback?: ProgressCallback, id?: number) => {
    if (callback) {
        return await MoveFile.mv_bulk(sourceFiles, destDir, callback, id);
    } else {
        return await MoveFile.mv_bulk(sourceFiles, destDir, id);
    }
};

export const mvSync = (sourceFile: string, destFile: string): number => {
    return MoveFile.mv_sync(sourceFile, destFile);
};

export const cancel = (id: number): boolean => {
    return MoveFile.cancel(id);
};

export const reserveCancellable = (): number => {
    return MoveFile.reserve_cancellable();
};

export const trash = (file: string): void => {
    return MoveFile.trash(file);
};

export const listVolumes = (): Volume[] => {
    return MoveFile.list_volumes();
};

export const getFileAttribute = (filePath: string): FileAttribute => {
    return MoveFile.get_file_attribute(filePath);
};
