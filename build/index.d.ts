declare namespace MoveFile {
    function mv(sourceFile:string, destFile:string, id?:number): Promise<void>;
    function mv(sourceFile:string, destFile:string, callback:Function, id?:number): Promise<void>;
    function mv_bulk(sourceFiles:string[], destDir:string, id?:number): Promise<void>;
    function mv_bulk(sourceFiles:string[], destDir:string, callback:Function, id?:number): Promise<void>;
    function mv_sync(sourceFile:string, destFile:string): number;
    function cancel(id:number):boolean;
    function reserve_cancellable():number;
    function trash(file:string):void;
    function list_volumes():any[];
    function get_file_attribute(filePath:string):any;
    function read_urls_from_clipboard(windowHandle:number):string[];
}

export = MoveFile;