declare namespace MoveFile {
    function mv(sourceFile:string, destFile:string, id?:number): Promise<void>;
    function mv(sourceFile:string, destFile:string, callback:Function, id?:number): Promise<void>;
    function mv_bulk(sourceFiles:string[], destDir:string, id?:number): Promise<void>;
    function mv_bulk(sourceFiles:string[], destDir:string, callback:Function, id?:number): Promise<void>;
    function mvSync(sourceFile:string, destFile:string): number;
    function cancel(id:number):boolean;
    function reserve():number;
    function trash(file:string):void;
}

export = MoveFile;