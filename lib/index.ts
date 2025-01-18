import * as MoveFile from "../build/index";

export type Progress = {
    totalFileSize: number;
    transferred: number;
};
export type ProgressCallback = (progress: Progress) => void;
export type Volume = {
    mountPoint: string;
    volumeLabel: string;
    availableUnits: number;
    totalUnits: number;
};
export type FileAttribute = {
    isDevice: boolean;
    isDirectory: boolean;
    isFile: boolean;
    isHidden: boolean;
    isReadOnly: boolean;
    isSymbolicLink: boolean;
    isSystem: boolean;
    atimeMs: number;
    ctimeMs: number;
    mtimeMs: number;
    birthtimeMs: number;
    size: number;
};

export type ClipboardOperation = "Copy" | "Move" | "None";
export type ClipboardData = {
    operation: ClipboardOperation;
    urls: string[];
};

export type Dirent = {
    name: string;
    parentPath: string;
    fullPath: string;
    attributes: FileAttribute;
    mimeType: string;
};

export class fs {
    static mv = async (sourceFile: string, destFile: string, callback?: ProgressCallback, id?: number) => {
        if (callback) {
            return await MoveFile.mv(sourceFile, destFile, callback, id);
        } else {
            return await MoveFile.mv(sourceFile, destFile);
        }
    };

    static mvAll = async (sourceFiles: string[], destDir: string, callback?: ProgressCallback, id?: number) => {
        if (callback) {
            return await MoveFile.mv_all(sourceFiles, destDir, callback, id);
        } else {
            return await MoveFile.mv_all(sourceFiles, destDir);
        }
    };

    static mvSync = (sourceFile: string, destFile: string): number => {
        return MoveFile.mv_sync(sourceFile, destFile);
    };

    static cancel = (id: number): boolean => {
        return MoveFile.cancel(id);
    };

    static reserveCancellable = (): number => {
        return MoveFile.reserve_cancellable();
    };

    static listVolumes = (): Volume[] => {
        return MoveFile.list_volumes();
    };

    static getFileAttribute = (filePath: string): FileAttribute => {
        return MoveFile.get_file_attribute(filePath);
    };

    static readdir = (directory: string, recursive: boolean, withMimeType: boolean): Dirent[] => {
        return MoveFile.readdir(directory, recursive, withMimeType);
    };

    static getMimeType = (filePath: string): string => {
        return MoveFile.get_mime_type(filePath);
    };
}

export class shell {
    static trash = (file: string): void => {
        return MoveFile.trash(file);
    };

    static openPath = (windowHandle: number, filePath: string) => {
        return MoveFile.open_path(windowHandle, filePath);
    };

    static openPathWith = (windowHandle: number, filePath: string) => {
        return MoveFile.open_path_with(windowHandle, filePath);
    };

    static openFileProperty = (windowHandle: number, filePath: string) => {
        return MoveFile.open_file_property(windowHandle, filePath);
    };

    static showItemInFolder = (filePath: string) => {
        return MoveFile.show_item_in_folder(filePath);
    };
}

export class clipboard {
    static isTextAvailable = () => {
        MoveFile.is_text_available();
    };

    static readText = (windowHandle: number): string => {
        return MoveFile.read_text(windowHandle);
    };

    static writeText = (windowHandle: number, text: string) => {
        return MoveFile.write_text(windowHandle, text);
    };

    static isUrisAvailable = () => {
        MoveFile.is_uris_available();
    };

    static readUris = (windowHandle: number): ClipboardData => {
        return MoveFile.read_uris(windowHandle);
    };

    static writeUris = (windowHandle: number, fullPaths: string[], operation: ClipboardOperation) => {
        return MoveFile.write_uris(windowHandle, fullPaths, operation);
    };
}

export class drag {
    static startDrag = (paths: string[], windowHandle?: number) => {
        return MoveFile.start_drag(paths, windowHandle ?? 0);
    };
}
