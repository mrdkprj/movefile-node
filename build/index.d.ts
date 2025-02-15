declare namespace MoveFile {
    function mv(from:string, to:string): void;
    function mv_all(from:string[], to:string): void;
    function copy(from:string, to:string):void;
    function copy_all(from:string[], to:string):void;
    function trash(file:string):void;
    function list_volumes():any[];
    function get_file_attribute(filePath:string):any;
    function open_path(fullPath:string):void;
    function open_path_with(fullPath:string, appPath:string):void;
    function open_file_property(fullPath:string):void;
    function show_item_in_folder(fullPath:string):void;
    function is_text_available():boolean;
    function read_text(windowHandle:number):string;
    function write_text(windowHandle:number, text:string):void;
    function is_uris_available():boolean;
    function read_uris(windowHandle:number):any;
    function write_uris(windowHandle:number, fullPaths:string[], operation:string):void;
    function readdir(directory:string, recursive:boolean, withMimeType:boolean):any[];
    function get_mime_type(filePath:string):string;
    function start_drag(paths:string[], windowHandle:number):void;
    function get_open_with(fullPath:string):any[];
    function show_open_with_dialog(fullPath:string):void;
    function register(windowHandle:number):void;
    function message():void;
    function sidecar():void;
}

export = MoveFile;