declare namespace MoveFile {
    function mv(sourceFile:string, destFile:string): number;
    function mv(sourceFile:string, destFile:string, callback:Function): number;
    function mvSync(sourceFile:string, destFile:string): number;
    function cancel(id:number):boolean;
}

export = MoveFile;