declare namespace MoveFile {
    function mv(sourceFile:string, destFile:string): Promise<void>;
    function mv(sourceFile:string, destFile:string, callback:Function, id?:number): Promise<void>;
    function mv_bulk(sourceFiles:string[], destDir:string): Promise<void>;
    function mv_bulk(sourceFiles:string[], destDir:string, callback:Function, id?:number): Promise<void>;
    function mv_sync(sourceFile:string, destFile:string): number;
    function cancel(id:number):boolean;
    function reserve_cancellable():number;
    function trash(file:string):void;
    function list_volumes():any[];
    function get_file_attribute(filePath:string):any;
    function open_path(windowHandle:number, fullPath:string):void;
    function open_file_property(windowHandle:number, fullPath:string):void;
    function read_text(windowHandle:number):string;
    function write_text(windowHandle:number, text:string):void;
    function read_urls_from_clipboard(windowHandle:number):any;
    function write_urls_to_clipboard(windowHandle:number, fullPaths:string[], operation:string):void;
}

export = MoveFile;